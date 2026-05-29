use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
use cpp2rust_demo::ast_parser;
use cpp2rust_demo::capture;
use cpp2rust_demo::error::Result;
use cpp2rust_demo::extractor;
use cpp2rust_demo::generator::hicc_codegen;
use cpp2rust_demo::generator::project_generator;
use cpp2rust_demo::layout::{self, FeatureLayout};
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
    /// Parse a .cpp2rust file and print the AST structure (for debugging)
    Parse(ParseArgs),
}

#[derive(Args)]
struct InitArgs {
    /// Feature name (default: "default")
    #[arg(long, default_value = "default")]
    feature: String,

    /// Skip files that fail later phases (stub in Phase T/0)
    #[arg(long)]
    skip_failed: bool,

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
    /// Feature name(s) to merge (default: "default"; can be specified multiple times)
    #[arg(long, default_value = "default")]
    feature: Vec<String>,

    /// Output feature name for the merged project (default: "merged")
    #[arg(long, default_value = "merged")]
    output: String,
}

#[derive(Args)]
struct ParseArgs {
    /// Path to the .cpp2rust file to parse
    file: std::path::PathBuf,
}

fn run_parse(args: ParseArgs) -> Result<()> {
    let file = &args.file;
    if !file.exists() {
        return Err(anyhow!("file not found: {}", file.display()));
    }
    println!("Parsing: {}", file.display());
    println!();
    let ast = ast_parser::parse_preprocessed(file)?;
    ast.print_tree();

    // 模板实例化统计
    let template_classes: Vec<_> = ast.classes.iter().filter(|c| !c.template_args.is_empty()).collect();
    if !template_classes.is_empty() {
        println!();
        println!("Template instantiations ({}):", template_classes.len());
        for tc in &template_classes {
            println!("  {} <{}>", tc.name, tc.template_args.join(", "));
        }
    }

    Ok(())
}

fn run_merge(args: MergeArgs) -> Result<()> {
    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);

    let features: Vec<String> = args.feature;
    let output_name = &args.output;

    println!("=== cpp2rust-demo merge ===");
    println!("Project root : {}", project_root.display());
    println!("Feature(s)   : {}", features.join(", "));
    println!("Output       : {}", output_name);
    println!();

    // 收集所有 feature 下的 unit .rs 文件
    let mut all_unit_paths: Vec<std::path::PathBuf> = Vec::new();

    for feature in &features {
        let feature_root = project_root.join(".cpp2rust").join(feature);
        if !feature_root.exists() {
            return Err(anyhow!(
                "feature '{}' not found at {}; run init first",
                feature,
                feature_root.display()
            ));
        }
        let src_dir = feature_root.join("rust").join("src");
        let unit_files = merger::collect_unit_rs_files(&src_dir);
        println!(
            "  Feature '{}': {} unit file(s) in {}",
            feature,
            unit_files.len(),
            src_dir.display()
        );
        all_unit_paths.extend(unit_files);
    }

    if all_unit_paths.is_empty() {
        println!("\nNo unit .rs files found. Run 'init' first.");
        return Ok(());
    }

    println!("\nMerging {} unit file(s)...", all_unit_paths.len());

    // 合并
    let merged_spec = merger::merge_units(&all_unit_paths, output_name);

    // 报告冲突
    if !merged_spec.conflicts.is_empty() {
        eprintln!("\n⚠ {} conflict(s) detected:", merged_spec.conflicts.len());
        for c in &merged_spec.conflicts {
            eprintln!("  {}", c);
        }
    }

    // 生成合并后的 Rust 代码
    let merged_rs = merger::emit_merged_rs(&merged_spec, output_name);

    // 写出到 .cpp2rust/<output>/rust/
    let output_layout = FeatureLayout::new(project_root.clone(), output_name);
    output_layout.create_dirs()?;

    // 写 lib.rs（合并内容）
    let src_dir = output_layout.rust_dir.join("src");
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| anyhow!("create src dir: {}", e))?;
    let lib_rs_path = src_dir.join("lib.rs");
    std::fs::write(&lib_rs_path, &merged_rs)
        .map_err(|e| anyhow!("write lib.rs: {}", e))?;

    // 写 Cargo.toml
    project_generator::write_cargo_toml(&output_layout.rust_dir, output_name)?;

    println!("\n\u{2713} cpp2rust-demo merge completed.");
    println!("\nMerge summary:");
    println!(
        "  {} class(es), {} fn binding(s), {} include line(s)",
        merged_spec.class_order.len(),
        merged_spec.fn_bindings.len(),
        merged_spec.cpp_lines.iter().filter(|l| l.contains("#include")).count(),
    );
    println!("\nOutput:");
    println!("  {}", lib_rs_path.display());
    println!("  {}", output_layout.rust_dir.join("Cargo.toml").display());

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
    println!("Skip failed  : {}", args.skip_failed);
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
    let mut parse_errors = 0usize;
    // 降级特性统计：tag → 出现次数
    let mut degraded_tags: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    // unit_path → 首次注册该路径的源文件（用于冲突诊断）
    let mut seen_unit_paths: std::collections::HashMap<String, std::path::PathBuf> = std::collections::HashMap::new();

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
                count_degraded_tags(&code, &mut degraded_tags);

                let elapsed_ms = file_start.elapsed().as_millis();
                println!(
                    "  {} \u{2192} {} class(es), {} fn(s), {} enum(s)  [{} ms]",
                    path.display(),
                    ast.classes.len(),
                    ast.functions.len(),
                    ast.enums.len(),
                    elapsed_ms,
                );

                project_generator::write_unit_rs(&lo.rust_dir, &unit_path, &code)?;
                unit_paths.push(unit_path);
            }
            Err(err) => {
                parse_errors += 1;
                let elapsed_ms = file_start.elapsed().as_millis();
                if args.skip_failed {
                    eprintln!(
                        "  Warning: parse failed for {} [{} ms]: {:#}",
                        path.display(), elapsed_ms, err
                    );
                    // 生成 stub 文件，避免 lib.rs 中模块声明缺失
                    let stub_code = format!(
                        "// cpp2rust-todo[PARSE_FAILED]: {}\n// Error: {:#}\n",
                        path.display(),
                        err
                    );
                    project_generator::write_unit_rs(&lo.rust_dir, &unit_path, &stub_code)?;
                    unit_paths.push(unit_path);
                } else {
                    return Err(err);
                }
            }
        }
    }

    if parse_errors > 0 {
        println!("\nWarning: {} file(s) failed to parse (--skip-failed active).", parse_errors);
    }

    // 降级特性汇总
    if !degraded_tags.is_empty() {
        println!("\n\u{26a0} Degraded features (require manual attention):");
        let mut tags: Vec<_> = degraded_tags.iter().collect();
        tags.sort_by_key(|(tag, _)| tag.as_str());
        for (tag, count) in &tags {
            println!("  [{}] \u{d7} {}", tag, count);
        }
        println!("  \u{2192} Search for 'cpp2rust-todo' in generated files to find these locations.");
    }

    // 生成 Cargo.toml 和 lib.rs（含中间 mod.rs）
    project_generator::write_cargo_toml(&lo.rust_dir, feature)?;
    project_generator::write_lib_rs(&lo.rust_dir, &unit_paths)?;

    println!("\n\u{2713} cpp2rust-demo init completed.");
    println!("\nOutput structure:");
    println!("  .cpp2rust/{}/", feature);
    println!("    \u{251c}\u{2500}\u{2500} c/          (captured .cpp2rust files, mirrors C project layout)");
    println!("    \u{251c}\u{2500}\u{2500} meta/       (build_cmd.txt, selected_files.json)");
    println!("    \u{2514}\u{2500}\u{2500} rust/       (generated Rust project: Cargo.toml, src/lib.rs, src/**/*.rs)");
    println!();
    println!("Generated {} unit file(s) in .cpp2rust/{}/rust/src/", unit_paths.len(), feature);
    if unit_paths.iter().any(|p| p.contains('/')) {
        println!("  (directory structure mirrors the C++ project layout)");
    }

    Ok(())
}

/// 扫描生成代码中的 `cpp2rust-todo[TAG]` 标签，统计各 tag 出现次数。
fn count_degraded_tags(code: &str, tags: &mut std::collections::HashMap<String, usize>) {
    for line in code.lines() {
        if let Some(start) = line.find("cpp2rust-todo[") {
            let rest = &line[start + "cpp2rust-todo[".len()..];
            if let Some(end) = rest.find(']') {
                let tag = rest[..end].to_string();
                *tags.entry(tag).or_insert(0) += 1;
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Init(args) => run_init(args),
        Commands::Merge(args) => run_merge(args),
        Commands::Parse(args) => run_parse(args),
    };
    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}
