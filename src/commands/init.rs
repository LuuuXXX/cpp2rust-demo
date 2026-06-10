//! `init` 子命令实现
//!
//! 执行编译拦截、AST 解析、代码生成全流程，将 C++ 项目转换为 hicc FFI 脚手架。

use anyhow::anyhow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use crate::ast_parser;
use crate::capture;
use crate::error::Result;
use crate::extractor;
use crate::ffi_model::FfiSpec;

/// `run_init` 内部使用的每单元数据，将第一趟解析结果传递到第二趟代码生成。
struct UnitData {
    unit_path: String,
    spec: FfiSpec,
}

use crate::generator::{hicc_codegen, project_generator};
use crate::layout::{self, FeatureLayout, InitReportData, InitUnitStat};
use crate::metrics::{count_file_lines, parse_todo_tag_from_line};
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

    print_capture_stats(&captured);

    let sel = InteractiveSelector;
    let selected = sel.select(&captured)?;
    println!("已为本 feature 选择 {} 个文件", selected.len());

    lo.save_selected_files(&selected)?;

    if selected.is_empty() {
        println!("未选择任何文件，跳过代码生成。");
        return Ok(());
    }

    println!("\n正在对选定文件运行 AST 解析与代码生成...");

    let (all_units, unit_stats) = first_pass_parse(&selected, &lo.c_dir, &project_root)?;
    let class_to_module = collect_class_map(&all_units);
    let (unit_paths, sorted_tags) = second_pass_generate(&all_units, &class_to_module, &lo.rust_dir)?;

    print_degraded_summary(&sorted_tags);

    // 生成 Cargo.toml、build.rs 和 lib.rs（含中间 mod.rs）
    project_generator::write_cargo_toml(&lo.rust_dir, feature)?;
    let lib_name = feature.replace('-', "_");
    project_generator::write_build_rs(&lo.rust_dir, &lib_name)?;
    project_generator::write_lib_rs(&lo.rust_dir, &unit_paths)?;

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
    println!("        └── src/        （lib.rs + 各编译单元 .rs 文件）");
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

/// 打印捕获的 `.cpp2rust` 文件行数统计（降序排列，最多显示 15 条）。
fn print_capture_stats(captured: &[PathBuf]) {
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

/// 第一趟：对所有选定文件执行 AST 解析与 FFI 提取。
///
/// 返回 `(all_units, unit_stats)`，其中 `all_units` 包含每个编译单元的 IR 数据，
/// `unit_stats` 用于写入 `init-report.md`。
///
/// 解析失败的文件记录为警告并跳过（收集式策略），流程继续处理剩余文件。
/// 若所有文件均解析失败，则返回错误，并汇总全部失败文件列表。
fn first_pass_parse(
    selected: &[PathBuf],
    c_dir: &std::path::Path,
    project_root: &std::path::Path,
) -> Result<(Vec<UnitData>, Vec<InitUnitStat>)> {
    let mut all_units: Vec<UnitData> = Vec::new();
    let mut unit_stats: Vec<InitUnitStat> = Vec::new();
    let mut seen_unit_paths: HashMap<String, PathBuf> = HashMap::new();
    let mut failed_files: Vec<String> = Vec::new();

    for path in selected {
        let file_start = Instant::now();

        // 从 `.cpp2rust` 路径推导原始 `.cpp` 路径
        // hook 命名规则：<c_dir>/<relative_from_project_root>.cpp2rust
        // 例：<c_dir>/src/foo.cpp.cpp2rust → project_root/src/foo.cpp
        let original_cpp = {
            let rel = path.strip_prefix(c_dir).unwrap_or(path.as_path());
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
        let unit_path = project_generator::derive_unit_path(c_dir, path);

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
                let msg = format!("{} [{} ms]: {:#}", path.display(), elapsed_ms, err);
                eprintln!("  警告：解析失败，已跳过 — {}", msg);
                failed_files.push(path.display().to_string());
            }
        }
    }

    // 若存在解析失败的文件，汇总打印但不中断（只要有成功的文件即可继续）
    if !failed_files.is_empty() {
        eprintln!(
            "\n⚠ {} 个文件解析失败（已跳过）：",
            failed_files.len()
        );
        for f in &failed_files {
            eprintln!("    ✗ {}", f);
        }
        // 若所有文件均失败，则返回错误，避免生成空项目
        if all_units.is_empty() {
            return Err(anyhow!(
                "全部 {} 个文件均处理失败:\n{}",
                failed_files.len(),
                failed_files.join("\n")
            ));
        }
    }

    Ok((all_units, unit_stats))
}

/// 建立跨模块类型映射：`class_name → 定义该类型的 unit_path`。
///
/// 只有实际生成了 `import_class!` 块的类（即 `ClassSpec::is_empty()` 为 false）才加入映射，
/// 与 `hicc_codegen::generate` 的跳过条件保持一致。
fn collect_class_map(all_units: &[UnitData]) -> HashMap<String, String> {
    let mut class_to_module: HashMap<String, String> = HashMap::new();
    for ud in all_units {
        for cs in ud.spec.class_specs.iter().filter(|cs| !cs.is_empty()) {
            if let Some(existing) = class_to_module.get(&cs.name) {
                eprintln!(
                    "  警告：类 '{}' 同时定义于 '{}' 和 '{}'；跨模块引用将使用第一个定义",
                    cs.name, existing, ud.unit_path
                );
            } else {
                class_to_module.insert(cs.name.clone(), ud.unit_path.clone());
            }
        }
    }
    class_to_module
}

/// 第二趟：生成代码（附加跨模块 `use` / opaque 声明）并写入文件，同时统计降级特性。
///
/// 返回 `(unit_paths, sorted_tags)`，其中 `sorted_tags` 是按 tag 字典序排列的
/// `(tag, Vec<(unit_path, count)>)` 列表，供打印摘要和写入报告使用。
fn second_pass_generate(
    all_units: &[UnitData],
    class_to_module: &HashMap<String, String>,
    rust_dir: &std::path::Path,
) -> Result<(Vec<String>, Vec<(String, Vec<(String, usize)>)>)> {
    let mut unit_paths: Vec<String> = Vec::new();
    let mut degraded_tags: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();

    for ud in all_units {
        let preamble = build_cross_module_preamble(&ud.spec, &ud.unit_path, class_to_module);
        let code = format!("{}{}", preamble, hicc_codegen::generate(&ud.spec));

        count_degraded_tags(&code, &ud.unit_path, &mut degraded_tags);

        project_generator::write_unit_rs(rust_dir, &ud.unit_path, &code)?;
        unit_paths.push(ud.unit_path.clone());
    }

    // BTreeMap 已保证 tag 字典序；内层 unit_path 同样需要排序
    let sorted_tags: Vec<(String, Vec<(String, usize)>)> = degraded_tags
        .into_iter()
        .map(|(tag, unit_map)| {
            let mut units: Vec<(String, usize)> = unit_map.into_iter().collect();
            units.sort_by(|a, b| a.0.cmp(&b.0));
            (tag, units)
        })
        .collect();

    Ok((unit_paths, sorted_tags))
}

/// 打印降级特性汇总（若无降级特性则静默）。
fn print_degraded_summary(sorted_tags: &[(String, Vec<(String, usize)>)]) {
    if sorted_tags.is_empty() {
        return;
    }
    println!("\n⚠ 降级特性（需要人工处理）：");
    for (tag, units) in sorted_tags {
        let total: usize = units.iter().map(|(_, c)| c).sum();
        println!("  [{}] × {} 次", tag, total);
        for (unit_path, count) in units {
            println!("      {} （{} 次）", unit_path, count);
        }
    }
    println!("  → 在生成文件中搜索 'cpp2rust-todo' 可定位这些位置。");
}

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
            "  警告：fwd_decl {:?} 格式不合法（单元 '{}'），期望格式为 'class TypeName;'",
            fwd_decl, unit_path
        );
        return None;
    }
    if !is_valid_identifier(type_name) {
        eprintln!(
            "  警告：fwd_decl {:?} 在单元 '{}' 中含无效标识符 '{}'，已跳过",
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
        .filter(|cs| !cs.is_empty())
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
    tags: &mut BTreeMap<String, BTreeMap<String, usize>>,
) {
    for line in code.lines() {
        if let Some(tag) = parse_todo_tag_from_line(line) {
            *tags
                .entry(tag.to_string())
                .or_default()
                .entry(unit_path.to_string())
                .or_insert(0) += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_valid_identifier ───────────────────

    #[test]
    fn valid_identifiers() {
        assert!(is_valid_identifier("Foo"));
        assert!(is_valid_identifier("foo_bar"));
        assert!(is_valid_identifier("_priv"));
        assert!(is_valid_identifier("FooBar123"));
        assert!(is_valid_identifier("a"));
    }

    #[test]
    fn invalid_identifiers_empty() {
        assert!(!is_valid_identifier(""));
    }

    #[test]
    fn invalid_identifiers_starts_with_digit() {
        assert!(!is_valid_identifier("1foo"));
        assert!(!is_valid_identifier("0_"));
    }

    #[test]
    fn invalid_identifiers_contain_special_chars() {
        assert!(!is_valid_identifier("foo-bar"));
        assert!(!is_valid_identifier("foo bar"));
        assert!(!is_valid_identifier("foo::bar"));
        assert!(!is_valid_identifier("foo.bar"));
    }

    // ── parse_fwd_decl ────────────────────────

    #[test]
    fn parse_fwd_decl_valid() {
        let result = parse_fwd_decl("class Foo;", "test");
        assert_eq!(result, Some("Foo"));
    }

    #[test]
    fn parse_fwd_decl_with_spaces() {
        // strip_prefix + strip_suffix + trim 应能处理带空格的形式
        let result = parse_fwd_decl("class  MyClass ;", "test");
        assert_eq!(result, Some("MyClass"));
    }

    #[test]
    fn parse_fwd_decl_empty_name_returns_none() {
        // "class ;" → 名称为空
        let result = parse_fwd_decl("class ;", "test");
        assert!(result.is_none());
    }

    #[test]
    fn parse_fwd_decl_invalid_identifier_returns_none() {
        // "class 1Foo;" → 非法标识符
        let result = parse_fwd_decl("class 1Foo;", "test");
        assert!(result.is_none());
    }

    #[test]
    fn parse_fwd_decl_wrong_format_returns_none() {
        // 不以 "class " 开头
        assert!(parse_fwd_decl("struct Foo;", "test").is_none());
        // 不以 ";" 结尾
        assert!(parse_fwd_decl("class Foo", "test").is_none());
    }

    // ── count_degraded_tags ───────────────────

    #[test]
    fn count_degraded_tags_basic() {
        let code = "// cpp2rust-todo[FP] some\n// cpp2rust-todo[OP] other\n// cpp2rust-todo[FP] again\n";
        let mut tags: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
        count_degraded_tags(code, "unit/foo", &mut tags);
        assert_eq!(tags["FP"]["unit/foo"], 2);
        assert_eq!(tags["OP"]["unit/foo"], 1);
    }

    #[test]
    fn count_degraded_tags_no_tags() {
        let code = "fn foo() {}\n// just a comment\n";
        let mut tags: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
        count_degraded_tags(code, "unit/bar", &mut tags);
        assert!(tags.is_empty());
    }

    #[test]
    fn count_degraded_tags_accumulates_across_units() {
        let code = "// cpp2rust-todo[VA] here\n";
        let mut tags: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
        count_degraded_tags(code, "unit/a", &mut tags);
        count_degraded_tags(code, "unit/b", &mut tags);
        assert_eq!(tags["VA"].len(), 2);
        assert_eq!(tags["VA"]["unit/a"], 1);
        assert_eq!(tags["VA"]["unit/b"], 1);
    }

    // ── print_capture_stats ──────────────────────
    // print_capture_stats 写入 stdout，测试边界值：空列表不应 panic

    #[test]
    fn print_capture_stats_empty_list() {
        // 传入空列表时应正常完成，不 panic
        print_capture_stats(&[]);
    }

    #[test]
    fn print_capture_stats_more_than_15() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        // 构造 20 个临时文件（内容各不相同以确保不同行数）
        let paths: Vec<PathBuf> = (0..20)
            .map(|i| {
                let p = tmp.path().join(format!("f{}.cpp2rust", i));
                // 写入 i+1 行内容
                std::fs::write(&p, "x\n".repeat(i + 1)).unwrap();
                p
            })
            .collect();
        // 超过 15 个文件时应正常完成，且只显示前 15 条
        print_capture_stats(&paths);
    }

    // ── build_cross_module_preamble ─────────────

    fn make_ffi_spec_with_fwd(fwd_decls: Vec<String>) -> FfiSpec {
        use crate::ffi_model::LibSpec;
        FfiSpec {
            unit_name: "test".to_string(),
            cpp_block_lines: vec![],
            class_specs: vec![],
            lib_spec: LibSpec {
                link_name: "test".to_string(),
                fwd_decls,
                fn_bindings: vec![],
            },
        }
    }

    /// 同模块类（class_specs 中已有该类，且 is_empty=false？不是，is_empty=true 时跳过）
    /// 空 ClassSpec（无方法/关联函数/destroy_fn）→ is_empty=true → 不生成 use，但也不生成 opaque
    #[test]
    fn cross_module_preamble_local_class_no_output() {
        // ClassSpec 有 methods 不为空，则不为空，local_class_names 含该类
        use crate::ffi_model::{ClassSpec, LibSpec, MethodBinding, SelfKind};
        let spec = FfiSpec {
            unit_name: "test".to_string(),
            cpp_block_lines: vec![],
            class_specs: vec![ClassSpec {
                name: "Foo".to_string(),
                methods: vec![MethodBinding {
                    cpp_sig: "void get()".to_string(),
                    rust_name: "get".to_string(),
                    self_kind: SelfKind::Ref,
                    params: vec![],
                    ret_type: None,
                    has_fn_ptr_param: false,
                }],
                associated_fns: vec![],
                destroy_fn: None,
                is_interface: false,
            }],
            lib_spec: LibSpec {
                link_name: "test".to_string(),
                fwd_decls: vec!["class Foo;".to_string()],
                fn_bindings: vec![],
            },
        };
        let mut class_to_module = HashMap::new();
        class_to_module.insert("Foo".to_string(), "unit/foo".to_string());

        // Foo 在本模块（class_specs 含 Foo 且非空），不生成任何 use/opaque
        let result = build_cross_module_preamble(&spec, "unit/foo", &class_to_module);
        assert!(result.is_empty(), "本模块已定义的类不应生成 use 或 opaque，got: {result:?}");
    }

    /// 他模块类（class_to_module 中有该类，且非本模块）→ 生成 `use crate::...::TypeName;`
    #[test]
    fn cross_module_preamble_other_module_generates_use() {
        let spec = make_ffi_spec_with_fwd(vec!["class Bar;".to_string()]);
        let mut class_to_module = HashMap::new();
        class_to_module.insert("Bar".to_string(), "unit/bar".to_string());

        let result = build_cross_module_preamble(&spec, "unit/foo", &class_to_module);
        assert!(
            result.contains("use crate::unit::bar::Bar;"),
            "他模块类应生成 use 语句，got: {result:?}"
        );
        assert!(!result.contains("import_class"), "不应生成 opaque import_class! 块，got: {result:?}");
    }

    /// 同 unit_path 的类（def_module == current_unit_path）→ 不生成 use
    #[test]
    fn cross_module_preamble_same_unit_no_use() {
        let spec = make_ffi_spec_with_fwd(vec!["class Baz;".to_string()]);
        let mut class_to_module = HashMap::new();
        // Baz 定义于同一模块
        class_to_module.insert("Baz".to_string(), "unit/foo".to_string());

        let result = build_cross_module_preamble(&spec, "unit/foo", &class_to_module);
        assert!(result.is_empty(), "同 unit_path 的类不应生成 use，got: {result:?}");
    }

    /// 未定义类（class_to_module 中无该类）→ 生成 opaque import_class! 块
    #[test]
    fn cross_module_preamble_undefined_class_generates_opaque() {
        let spec = make_ffi_spec_with_fwd(vec!["class Unknown;".to_string()]);
        let class_to_module = HashMap::new();

        let result = build_cross_module_preamble(&spec, "unit/foo", &class_to_module);
        assert!(
            result.contains("import_class!"),
            "未定义类应生成 opaque import_class! 块，got: {result:?}"
        );
        assert!(
            result.contains("Unknown"),
            "opaque 块应含类名，got: {result:?}"
        );
        assert!(!result.contains("use crate"), "不应生成 use 语句，got: {result:?}");
    }

    /// 多斜杠路径：路径中的 `/` 应转换为 `::` 生成嵌套模块路径
    #[test]
    fn cross_module_preamble_nested_path() {
        let spec = make_ffi_spec_with_fwd(vec!["class Nested;".to_string()]);
        let mut class_to_module = HashMap::new();
        class_to_module.insert("Nested".to_string(), "utils/helpers/nested".to_string());

        let result = build_cross_module_preamble(&spec, "unit/foo", &class_to_module);
        assert!(
            result.contains("use crate::utils::helpers::nested::Nested;"),
            "嵌套路径应正确转换为 Rust 模块路径，got: {result:?}"
        );
    }

    /// 非法 fwd_decl 格式（不以 "class " 开头）→ 跳过，返回空
    #[test]
    fn cross_module_preamble_invalid_fwd_decl_skipped() {
        let spec = make_ffi_spec_with_fwd(vec!["struct Foo;".to_string()]);
        let class_to_module = HashMap::new();

        let result = build_cross_module_preamble(&spec, "unit/foo", &class_to_module);
        assert!(result.is_empty(), "非法 fwd_decl 应被跳过，got: {result:?}");
    }

    /// 空 fwd_decls → 返回空字符串
    #[test]
    fn cross_module_preamble_no_fwd_decls() {
        let spec = make_ffi_spec_with_fwd(vec![]);
        let class_to_module = HashMap::new();

        let result = build_cross_module_preamble(&spec, "unit/foo", &class_to_module);
        assert!(result.is_empty(), "无 fwd_decls 时应返回空，got: {result:?}");
    }
}
