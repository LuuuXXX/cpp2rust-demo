//! `init` 子命令实现
//!
//! 执行编译拦截、AST 解析、代码生成全流程，将 C++ 项目转换为 hicc FFI 脚手架。

use anyhow::anyhow;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use crate::ast_parser;
use crate::capture;
use crate::error::Result;
use crate::extractor;
use crate::ffi_model::FfiSpec;
use crate::generator::{hicc_codegen, project_generator, smoke_test_gen};
use crate::layout::{self, FeatureLayout, InitReportData, InitUnitStat};
use crate::metrics::{count_file_lines, TODO_MARKER_PREFIX};
use crate::selector::{FileSelector, InteractiveSelector};

/// 执行 `init` 命令：编译拦截 → AST 解析 → 代码生成。
pub fn run_init(feature: &str, build_cmd: &[String]) -> Result<()> {
    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);

    println!("=== cpp2rust-demo init ===");
    println!("项目根目录 : {}", project_root.display());
    println!("Feature    : {}", feature);
    println!("构建命令   : {}", build_cmd.join(" "));
    println!();

    let lo = FeatureLayout::new(project_root.clone(), feature);
    lo.create_dirs()?;
    lo.save_build_cmd(build_cmd)?;

    let hook_so = capture::build_hook()?;
    capture::run_with_hook(&cwd, build_cmd, &project_root, &lo.feature_root, &hook_so)?;

    let captured = layout::scan_cpp2rust_files(&lo.c_dir)?;
    println!("\n已捕获 {} 个 .cpp2rust 文件", captured.len());

    if captured.is_empty() {
        println!("警告：未生成任何 .cpp2rust 文件。");
        println!("请确认构建命令确实编译了 C++ 文件。");
        return Ok(());
    }

    // ── 预处理文件行数统计 ─────────────────────────────────────────────────
    {
        use std::cmp::Reverse;
        let mut sizes: Vec<(&PathBuf, usize)> =
            captured.iter().map(|p| (p, count_file_lines(p))).collect();
        sizes.sort_by_key(|b| Reverse(b.1));
        let total: usize = sizes.iter().map(|(_, n)| n).sum();
        println!("\n── 捕获的 .cpp2rust 文件（行数，降序）──");
        for (path, lines) in sizes.iter().take(15) {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            println!("  {:>8} 行  {}", lines, name);
        }
        if sizes.len() > 15 {
            println!("  ...（共 {} 个文件，仅显示前 15 条）", sizes.len());
        }
        println!("  ────────────────────────────────────────");
        println!("  {:>8} 行  合计", total);
    }

    let sel = InteractiveSelector;
    let selected = sel.select(&captured)?;
    println!("已为本 feature 选择 {} 个文件", selected.len());

    lo.save_selected_files(&selected)?;

    if selected.is_empty() {
        println!("未选择任何文件，跳过代码生成。");
        return Ok(());
    }

    println!("\n正在对选定文件运行 AST 解析与代码生成...");
    let mut unit_stats: Vec<InitUnitStat> = Vec::new();
    // 降级特性统计：tag → (unit_path → 出现次数)
    let mut degraded_tags: HashMap<String, HashMap<String, usize>> = HashMap::new();
    // unit_path → 首次注册该路径的源文件（用于冲突诊断）
    let mut seen_unit_paths: HashMap<String, PathBuf> = HashMap::new();

    // ── 第一趟：解析所有文件，收集 (unit_path, spec, stats) ──────────────────
    struct UnitData {
        unit_path: String,
        spec: FfiSpec,
    }
    let mut all_units: Vec<UnitData> = Vec::new();

    for path in &selected {
        let file_start = Instant::now();

        // 从 `.cpp2rust` 路径推导原始 `.cpp` 路径
        // hook 命名规则：<c_dir>/<relative_from_project_root>.cpp2rust
        // 例：<c_dir>/src/foo.cpp.cpp2rust → project_root/src/foo.cpp
        let original_cpp = {
            let rel = path.strip_prefix(&lo.c_dir).unwrap_or(path.as_path());
            let rel_str = rel.to_string_lossy();
            let cpp_rel = rel_str
                .strip_suffix(".cpp2rust")
                .unwrap_or(&rel_str)
                .to_string();
            project_root.join(cpp_rel)
        };

        // unit_path = C++ 编译单元对应的 Rust 模块路径
        // 仅去掉首级 "src" 目录（避免双重 src），其余目录名（tests/、shim/ 等）完整保留
        // 例：<c_dir>/src/utils/foo.cpp.cpp2rust → "utils/foo"
        //     <c_dir>/tests/bar.cpp.cpp2rust     → "tests/bar"
        let unit_path = project_generator::derive_unit_path(&lo.c_dir, path);

        // 冲突检测：两个不同源文件映射到同一 unit_path，显示两个文件路径便于排查
        if let Some(first) = seen_unit_paths.get(&unit_path) {
            eprintln!(
                "  警告：单元路径冲突 '{}'：首次声明来自 {}，跳过 {}",
                unit_path,
                first.display(),
                path.display()
            );
            continue;
        }
        seen_unit_paths.insert(unit_path.clone(), path.clone());

        match ast_parser::parse_preprocessed(path) {
            Ok(ast) => {
                let (system_includes, project_header) =
                    extractor::read_source_includes(&original_cpp);
                let spec = extractor::extract(
                    &ast,
                    &unit_path,
                    &system_includes,
                    project_header.as_deref(),
                );

                let elapsed_ms = file_start.elapsed().as_millis();
                println!(
                    "  {} → {} 个类、{} 个函数、{} 个枚举  [{} ms]",
                    path.display(),
                    ast.classes.len(),
                    ast.functions.len(),
                    ast.enums.len(),
                    elapsed_ms,
                );

                unit_stats.push(InitUnitStat {
                    cpp2rust_path: path.display().to_string(),
                    unit_path: unit_path.clone(),
                    class_count: ast.classes.len(),
                    fn_count: ast.functions.len(),
                    enum_count: ast.enums.len(),
                    elapsed_ms,
                });

                all_units.push(UnitData { unit_path, spec });
            }
            Err(err) => {
                let elapsed_ms = file_start.elapsed().as_millis();
                return Err(anyhow!(
                    "parse failed for {} [{} ms]: {:#}",
                    path.display(),
                    elapsed_ms,
                    err
                ));
            }
        }
    }

    // ── 跨模块类型映射：class_name → 定义该类型的 unit_path ──────────────────
    // 只有实际生成了 import_class! 块的类（即不被 hicc_codegen 跳过的 ClassSpec）才加入映射。
    // 与 hicc_codegen::generate 的跳过条件保持一致：methods/associated_fns/destroy_fn 全空则跳过。
    let mut class_to_module: HashMap<String, String> = HashMap::new();
    for ud in &all_units {
        for cs in ud.spec.class_specs.iter().filter(|cs| {
            !(cs.methods.is_empty() && cs.associated_fns.is_empty() && cs.destroy_fn.is_none())
        }) {
            if let Some(existing) = class_to_module.get(&cs.name) {
                eprintln!(
                    "  警告：类 '{}' 同时定义于 '{}' 和 '{}'；\
跨模块引用将使用第一个定义",
                    cs.name, existing, ud.unit_path
                );
            } else {
                class_to_module.insert(cs.name.clone(), ud.unit_path.clone());
            }
        }
    }

    // ── 第二趟：生成代码（附加跨模块 use / opaque 声明）并写入文件 ──────────
    let mut unit_paths: Vec<String> = Vec::new();

    for ud in &all_units {
        let preamble = build_cross_module_preamble(&ud.spec, &ud.unit_path, &class_to_module);
        let code = format!("{}{}", preamble, hicc_codegen::generate(&ud.spec));

        // 统计降级特性（扫描生成代码中的 cpp2rust-todo 标签）
        count_degraded_tags(&code, &ud.unit_path, &mut degraded_tags);

        project_generator::write_unit_rs(&lo.rust_dir, &ud.unit_path, &code)?;
        unit_paths.push(ud.unit_path.clone());
    }

    // 降级特性汇总
    let mut sorted_tags: Vec<(String, Vec<(String, usize)>)> = degraded_tags
        .into_iter()
        .map(|(tag, unit_map)| {
            let mut units: Vec<(String, usize)> = unit_map.into_iter().collect();
            units.sort_by(|a, b| a.0.cmp(&b.0));
            (tag, units)
        })
        .collect();
    sorted_tags.sort_by(|a, b| a.0.cmp(&b.0));
    if !sorted_tags.is_empty() {
        println!("\n⚠ 降级特性（需要人工处理）：");
        for (tag, units) in &sorted_tags {
            let total: usize = units.iter().map(|(_, c)| c).sum();
            println!("  [{}] × {} 次", tag, total);
            for (unit_path, count) in units {
                println!("      {} （{} 次）", unit_path, count);
            }
        }
        println!("  → 在生成文件中搜索 'cpp2rust-todo' 可定位这些位置。");
    }

    // 生成 Cargo.toml、build.rs 和 lib.rs（含中间 mod.rs）
    project_generator::write_cargo_toml(&lo.rust_dir, feature)?;
    let lib_name = feature.replace('-', "_");
    project_generator::write_build_rs(&lo.rust_dir, &lib_name)?;
    project_generator::write_lib_rs(&lo.rust_dir, &unit_paths)?;

    // 生成 tests/smoke_test.rs（冒烟测试）
    let smoke_units: Vec<(&str, &FfiSpec)> = all_units
        .iter()
        .map(|ud| (ud.unit_path.as_str(), &ud.spec))
        .collect();
    let smoke_content = smoke_test_gen::generate(&smoke_units, &lib_name);
    project_generator::write_smoke_test(&lo.rust_dir, &smoke_content)?;

    // 生成 meta/init-report.md
    let report_data = InitReportData {
        feature,
        build_cmd: &build_cmd.join(" "),
        captured_count: captured.len(),
        selected_count: selected.len(),
        units: &unit_stats,
        degraded_tags: &sorted_tags,
    };
    lo.save_init_report(&report_data)?;

    println!("\n✓ cpp2rust-demo init 完成。");
    println!("\n输出目录结构:");
    println!("  .cpp2rust/{}/", feature);
    println!("    ├── c/          （捕获的 .cpp2rust 文件，目录结构与 C++ 项目一致）");
    println!("    ├── meta/       （build_cmd.txt、selected_files.json、init-report.md）");
    println!("    └── rust/       （生成的 Rust 项目）");
    println!("        ├── Cargo.toml");
    println!("        ├── build.rs");
    println!("        ├── src/        （lib.rs + 各编译单元 .rs 文件）");
    println!("        └── tests/smoke_test.rs  （FFI 冒烟测试）");
    println!();
    println!(
        "已在 .cpp2rust/{}/rust/src/ 生成 {} 个单元文件",
        feature,
        unit_paths.len()
    );
    if unit_paths.iter().any(|p| p.contains('/')) {
        println!("  （目录结构与 C++ 项目一致）");
    }
    println!(
        "  → 运行 'cpp2rust-demo merge --feature {}' 整理输出结构。",
        feature
    );

    Ok(())
}

// ─── 内部辅助函数 ─────────────────────────────────────────────────────────────

/// 为无任何模块定义的 C typedef struct 生成 `hicc::import_class!` opaque 声明块，
/// 使该类型自动实现 `AbiClass`，满足 `import_lib!` 中 `class TypeName;` 的 trait 约束。
fn opaque_import_class_block(type_name: &str) -> String {
    format!(
        "hicc::import_class! {{\n    #[cpp(class = \"{n}\")]\n    pub class {n} {{}}\n}}\n",
        n = type_name
    )
}

/// 返回 `true` 当且仅当 `s` 是合法的 C++/Rust 标识符（ASCII 字母、数字、下划线，首字符非数字）。
fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// 若 `fwd_decl` 为 `"class TypeName;"` 形式，则返回 `TypeName`。
/// 若格式不合法或标识符无效，则输出警告并返回 `None`。
fn parse_fwd_decl<'a>(fwd_decl: &'a str, unit_path: &str) -> Option<&'a str> {
    let type_name = fwd_decl
        .strip_prefix("class ")
        .and_then(|s| s.strip_suffix(';'))
        .map(str::trim)
        .unwrap_or("");

    if type_name.is_empty() {
        eprintln!(
            "  Warning: malformed fwd_decl {:?} in unit '{}'; expected format 'class TypeName;'",
            fwd_decl, unit_path
        );
        return None;
    }
    if !is_valid_identifier(type_name) {
        eprintln!(
            "  Warning: fwd_decl {:?} in unit '{}' contains an invalid identifier '{}'; skipping",
            fwd_decl, unit_path, type_name
        );
        return None;
    }
    Some(type_name)
}

/// 为每个编译单元生成跨模块类型引用前缀。
///
/// 当 `import_lib!` 块引用的类型在其他模块由 `import_class!` 定义时，
/// 生成对应的 `use crate::...::TypeName;` 语句。
/// 若类型未在任何模块定义（如 C typedef struct），则在本模块生成 opaque 类型声明。
fn build_cross_module_preamble(
    spec: &FfiSpec,
    current_unit_path: &str,
    class_to_module: &HashMap<String, String>,
) -> String {
    // 只计入实际生成了 import_class! 块的类（与 hicc_codegen::generate 的跳过条件一致）
    let local_class_names: HashSet<&str> = spec
        .class_specs
        .iter()
        .filter(|cs| {
            !(cs.methods.is_empty() && cs.associated_fns.is_empty() && cs.destroy_fn.is_none())
        })
        .map(|cs| cs.name.as_str())
        .collect();

    let mut use_imports = String::new();
    let mut opaque_decls = String::new();

    for fwd_decl in &spec.lib_spec.fwd_decls {
        let type_name = match parse_fwd_decl(fwd_decl, current_unit_path) {
            Some(n) => n,
            None => continue,
        };

        if local_class_names.contains(type_name) {
            continue;
        }

        if let Some(def_module) = class_to_module.get(type_name) {
            if def_module != current_unit_path {
                let module_path = def_module.replace('/', "::");
                use_imports.push_str(&format!("use crate::{}::{};\n", module_path, type_name));
            }
        } else {
            opaque_decls.push_str(&opaque_import_class_block(type_name));
        }
    }

    if use_imports.is_empty() && opaque_decls.is_empty() {
        String::new()
    } else {
        format!("{}{}\n", use_imports, opaque_decls)
    }
}

/// 扫描生成代码中的 `cpp2rust-todo[TAG]` 标签，按编译单元统计各 tag 出现次数。
fn count_degraded_tags(
    code: &str,
    unit_path: &str,
    tags: &mut HashMap<String, HashMap<String, usize>>,
) {
    for line in code.lines() {
        if let Some(start) = line.find(TODO_MARKER_PREFIX) {
            let rest = &line[start + TODO_MARKER_PREFIX.len()..];
            if let Some(end) = rest.find(']') {
                let tag = rest[..end].to_string();
                *tags
                    .entry(tag)
                    .or_default()
                    .entry(unit_path.to_string())
                    .or_insert(0) += 1;
            }
        }
    }
}
