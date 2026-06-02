use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
use cpp2rust_demo::ast_parser;
use cpp2rust_demo::capture;
use cpp2rust_demo::error::Result;
use cpp2rust_demo::extractor;
use cpp2rust_demo::ffi_model::FfiSpec;
use cpp2rust_demo::generator::hicc_codegen;
use cpp2rust_demo::generator::project_generator;
use cpp2rust_demo::layout::{self, FeatureLayout, InitReportData, InitUnitStat, MergeReportData};
use cpp2rust_demo::merger;
use cpp2rust_demo::selector::{FileSelector, InteractiveSelector};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "cpp2rust-demo")]
#[command(about = "Minimal C++-to-Rust workflow: build capture + Rust scaffolding")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Capture a C++ build and prepare Rust scaffolding inputs
    Init(InitArgs),
    /// Merge generated per-symbol outputs into module-level files
    Merge(MergeArgs),
}

#[derive(Args)]
struct InitArgs {
    /// Feature name (default: "default")
    #[arg(long, default_value = "default")]
    feature: String,

    /// Build command to execute (use after '--')
    /// Example: cpp2rust-demo init -- make -j4
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        required = true,
        value_name = "BUILD_CMD"
    )]
    build_cmd: Vec<String>,
}

#[derive(Args)]
struct MergeArgs {
    /// Feature name to merge (default: "default")
    #[arg(long, default_value = "default")]
    feature: String,
}

fn run_merge(args: MergeArgs) -> Result<()> {
    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);

    let feature = &args.feature;

    println!("=== cpp2rust-demo merge ===");
    println!("Project root : {}", project_root.display());
    println!("Feature      : {}", feature);
    println!();

    let lo = FeatureLayout::new(project_root.clone(), feature);
    if !lo.feature_root.exists() {
        return Err(anyhow!(
            "feature '{}' not found at {}; run init first",
            feature,
            lo.feature_root.display()
        ));
    }

    // 确定 canonical 来源（src.1 优先，否则取 src 实际目录）
    let canonical_src = if lo.rust_dir.join("src.1").is_dir() {
        lo.rust_dir.join("src.1")
    } else {
        lo.rust_dir.join("src")
    };

    if !canonical_src.exists() {
        return Err(anyhow!(
            "rust/src not found under {}; run init first",
            lo.rust_dir.display()
        ));
    }

    let unit_files = merger::collect_unit_rs_files(&canonical_src);
    println!(
        "  Feature '{}': {} unit file(s) in {}",
        feature,
        unit_files.len(),
        canonical_src.display()
    );

    if unit_files.is_empty() {
        println!("\nNo unit .rs files found. Run 'init' first.");
        return Ok(());
    }

    println!("\nMerging {} unit file(s)...", unit_files.len());

    merger::merge_in_place(&lo.rust_dir)?;

    // 生成 meta/merge-report.md
    let report_data = MergeReportData {
        feature,
        unit_count: unit_files.len(),
        conflicts: &[],
    };
    lo.save_merge_report(&report_data)?;

    println!("\n\u{2713} cpp2rust-demo merge completed.");
    println!("\nOutput:");
    println!("  .cpp2rust/{}/", feature);
    println!("    \u{251c}\u{2500}\u{2500} meta/");
    println!("    \u{2502}   \u{2514}\u{2500}\u{2500} merge-report.md  (merge summary)");
    println!("    \u{2514}\u{2500}\u{2500} rust/");
    println!("        \u{251c}\u{2500}\u{2500} src.1/  (init \u{8f93}\u{51fa}\u{5907}\u{4efd})");
    println!(
        "        \u{251c}\u{2500}\u{2500} src.2/  (merge \u{8f93}\u{51fa}\u{ff0c}\u{76ee}\u{5f55}\u{7ed3}\u{6784}\u{4e0e} C++ \u{9879}\u{76ee}\u{4e00}\u{81f4})"
    );
    println!("        \u{2514}\u{2500}\u{2500} src     (symlink \u{2192} src.2)");

    Ok(())
}

fn run_init(args: InitArgs) -> Result<()> {
    let feature = &args.feature;
    let build_cmd = &args.build_cmd;

    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);

    println!("=== cpp2rust-demo init ===");
    println!("Project root : {}", project_root.display());
    println!("Feature      : {}", feature);
    println!("Build command: {}", build_cmd.join(" "));
    println!();

    let lo = FeatureLayout::new(project_root.clone(), feature);
    lo.create_dirs()?;
    lo.save_build_cmd(build_cmd)?;

    let hook_so = capture::build_hook()?;
    capture::run_with_hook(&cwd, build_cmd, &project_root, &lo.feature_root, &hook_so)?;

    let captured = layout::scan_cpp2rust_files(&lo.c_dir)?;
    println!("\nCaptured {} .cpp2rust file(s)", captured.len());

    if captured.is_empty() {
        println!("Warning: no .cpp2rust files were generated.");
        println!("Make sure your build command compiles C++ files.");
        return Ok(());
    }

    let sel = InteractiveSelector;
    let selected = sel.select(&captured)?;
    println!("{} file(s) selected for this feature", selected.len());

    lo.save_selected_files(&selected)?;

    if selected.is_empty() {
        println!("No files selected – skipping code generation.");
        return Ok(());
    }

    println!("\nRunning AST parser and code generation on selected files...");
    let mut unit_stats: Vec<InitUnitStat> = Vec::new();
    // 降级特性统计：tag → (unit_path → 出现次数)
    let mut degraded_tags: HashMap<String, HashMap<String, usize>> = HashMap::new();
    // unit_path → 首次注册该路径的源文件（用于冲突诊断）
    let mut seen_unit_paths: HashMap<String, std::path::PathBuf> = HashMap::new();

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

        // unit_path = C++ 编译单元对应的 Rust 模块路径（去掉首级目录，避免双重 src）
        // 例：<c_dir>/src/utils/foo.cpp.cpp2rust → "utils/foo"
        let unit_path = project_generator::derive_unit_path(&lo.c_dir, path);

        // 冲突检测：两个不同源文件映射到同一 unit_path，显示两个文件路径便于排查
        if let Some(first) = seen_unit_paths.get(&unit_path) {
            eprintln!(
                "  Warning: unit path conflict '{}': first claimed by {}, now skipping {}",
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
                    "  {} \u{2192} {} class(es), {} fn(s), {} enum(s)  [{} ms]",
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

                all_units.push(UnitData {
                    unit_path,
                    spec,
                });
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
    let class_to_module: HashMap<String, String> = all_units
        .iter()
        .flat_map(|ud| {
            ud.spec
                .class_specs
                .iter()
                .filter(|cs| {
                    !(cs.methods.is_empty()
                        && cs.associated_fns.is_empty()
                        && cs.destroy_fn.is_none())
                })
                .map(move |cs| (cs.name.clone(), ud.unit_path.clone()))
        })
        .collect();

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
        println!("\n\u{26a0} 降级特性（需要人工处理）：");
        for (tag, units) in &sorted_tags {
            let total: usize = units.iter().map(|(_, c)| c).sum();
            println!("  [{}] \u{d7} {} 次", tag, total);
            for (unit_path, count) in units {
                println!("      {} （{} 次）", unit_path, count);
            }
        }
        println!(
            "  \u{2192} 在生成文件中搜索 'cpp2rust-todo' 可定位这些位置。"
        );
    }

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

    println!("\n\u{2713} cpp2rust-demo init completed.");
    println!("\nOutput structure:");
    println!("  .cpp2rust/{}/", feature);
    println!(
        "    \u{251c}\u{2500}\u{2500} c/          (captured .cpp2rust files, mirrors C++ project layout)"
    );
    println!(
        "    \u{251c}\u{2500}\u{2500} meta/       (build_cmd.txt, selected_files.json, init-report.md)"
    );
    println!(
        "    \u{2514}\u{2500}\u{2500} rust/       (generated Rust project: Cargo.toml, src/lib.rs, src/**/*.rs)"
    );
    println!();
    println!(
        "Generated {} unit file(s) in .cpp2rust/{}/rust/src/",
        unit_paths.len(),
        feature
    );
    if unit_paths.iter().any(|p| p.contains('/')) {
        println!("  (directory structure mirrors the C++ project layout)");
    }

    Ok(())
}

/// 为每个编译单元生成跨模块类型引用前缀。
///
/// 当 `import_lib!` 块引用的类型在其他模块由 `import_class!` 定义时，
/// 生成对应的 `use crate::...::TypeName;` 语句。
/// 若类型未在任何模块定义（如 C typedef struct），则在本模块生成 opaque 类型声明，
/// 以便 `import_lib!` 宏展开时可以找到该类型。
fn build_cross_module_preamble(
    spec: &FfiSpec,
    current_unit_path: &str,
    class_to_module: &HashMap<String, String>,
) -> String {
    // 只计入实际生成了 import_class! 块的类（与 hicc_codegen::generate 的跳过条件一致）
    let local_class_names: std::collections::HashSet<&str> = spec
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
        // fwd_decl 格式：`"class TypeName;"`
        let type_name = fwd_decl
            .strip_prefix("class ")
            .and_then(|s| s.strip_suffix(';'))
            .unwrap_or("")
            .trim();

        if type_name.is_empty() || local_class_names.contains(type_name) {
            // 本模块已有 import_class! 定义，无需额外引入
            continue;
        }

        if let Some(def_module) = class_to_module.get(type_name) {
            if def_module != current_unit_path {
                // 类型由其他模块的 import_class! 定义 → 生成 use 导入
                let module_path = def_module.replace('/', "::");
                use_imports.push_str(&format!("use crate::{}::{};\n", module_path, type_name));
            }
        } else {
            // 无任何模块拥有该类型（如 C typedef struct）→ 用 import_class! 声明为 opaque 类型，
            // 使其自动实现 AbiClass，满足 import_lib! 中 class TypeName; 的 trait 约束。
            opaque_decls.push_str(&format!(
                "hicc::import_class! {{\n    #[cpp(class = \"{n}\")]\n    pub class {n} {{}}\n}}\n",
                n = type_name
            ));
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
    tags: &mut std::collections::HashMap<String, std::collections::HashMap<String, usize>>,
) {
    for line in code.lines() {
        if let Some(start) = line.find("cpp2rust-todo[") {
            let rest = &line[start + "cpp2rust-todo[".len()..];
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

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Init(args) => run_init(args),
        Commands::Merge(args) => run_merge(args),
    };
    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_help_does_not_panic() {
        Cli::command().debug_assert();
    }

    #[test]
    fn merge_default_feature() {
        let args = Cli::try_parse_from(["cpp2rust-demo", "merge"]).unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        assert_eq!(merge.feature, "default");
    }

    #[test]
    fn merge_custom_feature() {
        let args =
            Cli::try_parse_from(["cpp2rust-demo", "merge", "--feature", "core_lib"]).unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        assert_eq!(merge.feature, "core_lib");
    }

    #[test]
    fn init_default_feature() {
        let args = Cli::try_parse_from(["cpp2rust-demo", "init", "--", "make"]).unwrap();
        let Commands::Init(init) = args.command else {
            panic!("expected Init");
        };
        assert_eq!(init.feature, "default");
        assert_eq!(init.build_cmd, vec!["make"]);
    }

    #[test]
    fn init_requires_build_cmd() {
        let result = Cli::try_parse_from(["cpp2rust-demo", "init"]);
        assert!(result.is_err());
    }
}
