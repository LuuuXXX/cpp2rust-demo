use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
use cpp2rust_demo::ast_parser;
use cpp2rust_demo::capture;
use cpp2rust_demo::error::Result;
use cpp2rust_demo::extractor;
use cpp2rust_demo::generator::hicc_codegen;
use cpp2rust_demo::generator::project_generator;
use cpp2rust_demo::layout::{
    self, CrossMergeReportData, FeatureLayout, InitReportData, InitUnitStat, MergeReportData,
};
use cpp2rust_demo::merger;
use cpp2rust_demo::selector::{FileSelector, InteractiveSelector};
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
    /// Feature 名称，可重复指定多次（默认："default"）。
    /// 指定一个时：与原有行为一致，将该 feature 的 init 输出整理备份。
    /// 指定多个时：将各 feature 的编译单元跨 feature 合并，输出到以下划线拼接的新目录，
    /// 例如 `--feature a --feature b` → `.cpp2rust/a_b/`。
    #[arg(long = "feature", value_name = "NAME")]
    features: Vec<String>,
}

fn run_merge(args: MergeArgs) -> Result<()> {
    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);

    // 单 feature 模式（0 或 1 个 feature）
    let features = if args.features.is_empty() {
        vec!["default".to_string()]
    } else {
        args.features
    };

    if features.len() == 1 {
        run_merge_single(project_root, &features[0])
    } else {
        run_merge_cross(project_root, &features)
    }
}

/// 单 feature merge：与原有逻辑完全一致。
fn run_merge_single(project_root: std::path::PathBuf, feature: &str) -> Result<()> {
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

/// 多 feature 跨 feature 合并：将多个 feature 的编译单元聚合到新目录。
fn run_merge_cross(project_root: std::path::PathBuf, features: &[String]) -> Result<()> {
    let merged_name = features.join("_");

    println!("=== cpp2rust-demo merge (跨 feature 合并) ===");
    println!("Project root  : {}", project_root.display());
    println!("Source features: {}", features.join(", "));
    println!("Output feature : {}", merged_name);
    println!();

    // 验证各 source feature 目录存在，收集 unit 文件
    let mut all_unit_files: Vec<std::path::PathBuf> = Vec::new();
    for feature in features {
        let lo = FeatureLayout::new(project_root.clone(), feature);
        if !lo.feature_root.exists() {
            return Err(anyhow!(
                "feature '{}' not found at {}; run 'init --feature {}' first",
                feature,
                lo.feature_root.display(),
                feature,
            ));
        }
        let unit_files = merger::collect_feature_unit_rs_files(&lo);
        println!(
            "  Feature '{}': {} unit file(s)",
            feature,
            unit_files.len()
        );
        all_unit_files.extend(unit_files);
    }

    if all_unit_files.is_empty() {
        println!("\nNo unit .rs files found in any source feature. Run 'init' first.");
        return Ok(());
    }

    println!(
        "\nMerging {} unit file(s) across {} features...",
        all_unit_files.len(),
        features.len()
    );

    // 跨 feature 聚合（去重 + 冲突检测）
    let spec = merger::merge_units(&all_unit_files);

    if !spec.conflicts.is_empty() {
        println!("\n\u{26a0} {} 个冲突（详见 merge-report.md）：", spec.conflicts.len());
        for c in &spec.conflicts {
            println!("  - {}", c.lines().next().unwrap_or(c));
        }
    }

    // 创建 output feature 目录并写出 Rust 项目
    let out_lo = FeatureLayout::new(project_root.clone(), &merged_name);
    out_lo.create_dirs()?;

    // lib_name：Rust 标识符形式（将 `-` 替换为 `_`），用于 build.rs 中的 compile() 调用
    let lib_name = merged_name.replace('-', "_");

    // 生成合并后的单一 ffi.rs（所有 symbol 合并到一个文件）
    let ffi_code = merger::emit_merged_rs(&spec, &lib_name);
    let unit_paths = vec!["ffi".to_string()];
    project_generator::write_unit_rs(&out_lo.rust_dir, "ffi", &ffi_code)?;
    project_generator::write_cargo_toml(&out_lo.rust_dir, &merged_name)?;
    project_generator::write_build_rs(&out_lo.rust_dir, &lib_name)?;
    project_generator::write_lib_rs(&out_lo.rust_dir, &unit_paths)?;

    // 生成 meta/merge-report.md
    let report_data = CrossMergeReportData {
        source_features: features,
        merged_name: &merged_name,
        unit_count: all_unit_files.len(),
        conflicts: &spec.conflicts,
    };
    out_lo.save_cross_merge_report(&report_data)?;

    println!("\n\u{2713} cpp2rust-demo merge (跨 feature 合并) completed.");
    println!("\nOutput:");
    println!("  .cpp2rust/{}/", merged_name);
    println!("    \u{251c}\u{2500}\u{2500} meta/");
    println!("    \u{2502}   \u{2514}\u{2500}\u{2500} merge-report.md  (合并摘要)");
    println!("    \u{2514}\u{2500}\u{2500} rust/");
    println!("        \u{251c}\u{2500}\u{2500} Cargo.toml");
    println!("        \u{251c}\u{2500}\u{2500} build.rs");
    println!("        \u{2514}\u{2500}\u{2500} src/");
    println!("            \u{251c}\u{2500}\u{2500} lib.rs");
    println!("            \u{2514}\u{2500}\u{2500} ffi.rs   (所有 feature 的合并 FFI 代码)");

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
    let mut unit_paths: Vec<String> = Vec::new();
    let mut unit_stats: Vec<InitUnitStat> = Vec::new();
    // 降级特性统计：tag → (unit_path → 出现次数)
    let mut degraded_tags: std::collections::HashMap<
        String,
        std::collections::HashMap<String, usize>,
    > = std::collections::HashMap::new();
    // unit_path → 首次注册该路径的源文件（用于冲突诊断）
    let mut seen_unit_paths: std::collections::HashMap<String, std::path::PathBuf> =
        std::collections::HashMap::new();

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
                let code = hicc_codegen::generate(&spec);

                // 统计降级特性（扫描生成代码中的 cpp2rust-todo 标签）
                count_degraded_tags(&code, &unit_path, &mut degraded_tags);

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

                project_generator::write_unit_rs(&lo.rust_dir, &unit_path, &code)?;
                unit_paths.push(unit_path);
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
    fn merge_no_feature_defaults_to_default() {
        // 不传 --feature 时，features 为空向量；run_merge 内部自动补充 "default"
        let args = Cli::try_parse_from(["cpp2rust-demo", "merge"]).unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        assert!(merge.features.is_empty(), "no --feature → empty vec");
    }

    #[test]
    fn merge_single_feature() {
        let args =
            Cli::try_parse_from(["cpp2rust-demo", "merge", "--feature", "core_lib"]).unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        assert_eq!(merge.features, vec!["core_lib"]);
    }

    #[test]
    fn merge_multi_feature_computes_merged_name() {
        let args = Cli::try_parse_from([
            "cpp2rust-demo",
            "merge",
            "--feature",
            "linux_x86",
            "--feature",
            "arm",
        ])
        .unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        assert_eq!(merge.features, vec!["linux_x86", "arm"]);
        // 验证拼接逻辑
        assert_eq!(merge.features.join("_"), "linux_x86_arm");
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
