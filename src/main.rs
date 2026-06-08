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
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

// ─── post-merge FFI 统计 ───────────────────────────────────────────────────

struct RustSrcMetrics {
    rs_files: Vec<PathBuf>,
    import_lib_files: usize,
    import_class_files: usize,
    fn_binding_count: usize,
    /// link_name 值中含路径分隔符 '/' 的列表
    bad_link_names: Vec<String>,
    /// `#include` 指令总数
    include_count: usize,
    /// cpp2rust-todo 降级标记总数
    todo_count: usize,
    /// (tag, total_count) 降级标记按 tag 汇总
    degraded_tags: Vec<(String, usize)>,
}

/// 统计文件行数（逐行读取，内存高效）。
fn count_file_lines(path: &Path) -> usize {
    std::fs::File::open(path)
        .map(|f| BufReader::new(f).lines().count())
        .unwrap_or(0)
}

/// 扫描 `rust_src` 目录下所有 `.rs` 文件，统计 FFI 绑定指标。
fn collect_rust_src_metrics(rust_src: &Path) -> RustSrcMetrics {
    let mut rs_files: Vec<PathBuf> = WalkDir::new(rust_src)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        .map(|e| e.path().to_path_buf())
        .collect();
    rs_files.sort();

    let mut import_lib_files = 0usize;
    let mut import_class_files = 0usize;
    let mut fn_binding_count = 0usize;
    let mut bad_link_names: Vec<String> = Vec::new();
    let mut include_count = 0usize;
    let mut todo_tags: HashMap<String, usize> = HashMap::new();

    for path in &rs_files {
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        if content.contains("hicc::import_lib!") {
            import_lib_files += 1;
        }
        if content.contains("hicc::import_class!") {
            import_class_files += 1;
        }
        fn_binding_count += content.matches("#[cpp(func =").count();

        for line in content.lines() {
            let trimmed = line.trim();
            // 仅统计行首 #include（trimmed 以 "#include" 开头即不是注释行）
            if trimmed.starts_with("#include") {
                include_count += 1;
            }
            // link_name = "..." 提取
            if let Some(pos) = trimmed.find("link_name = \"") {
                let rest = &trimmed[pos + "link_name = \"".len()..];
                if let Some(end) = rest.find('"') {
                    let name = &rest[..end];
                    if name.contains('/') {
                        bad_link_names.push(name.to_string());
                    }
                }
            }
            // cpp2rust-todo[TAG] 统计
            if let Some(start) = line.find("cpp2rust-todo[") {
                let rest = &line[start + "cpp2rust-todo[".len()..];
                if let Some(end) = rest.find(']') {
                    let tag = rest[..end].to_string();
                    *todo_tags.entry(tag).or_insert(0) += 1;
                }
            }
        }
    }

    let mut degraded_tags: Vec<(String, usize)> = todo_tags.into_iter().collect();
    degraded_tags.sort_by(|a, b| a.0.cmp(&b.0));
    let todo_count: usize = degraded_tags.iter().map(|(_, c)| c).sum();

    RustSrcMetrics {
        rs_files,
        import_lib_files,
        import_class_files,
        fn_binding_count,
        bad_link_names,
        include_count,
        todo_count,
        degraded_tags,
    }
}

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
    /// 捕获 C++ 构建过程并准备 Rust 脚手架输入
    Init(InitArgs),
    /// 将每个符号生成的输出合并到模块级文件
    Merge(MergeArgs),
}

#[derive(Args)]
struct InitArgs {
    /// 特性名称（默认："default"）
    #[arg(long, default_value = "default")]
    feature: String,

    /// 要执行的构建命令（置于 '--' 之后）
    /// 示例：cpp2rust-demo init -- make -j4
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
    /// 要合并的特性名称（默认："default"；可多次指定以合并多个 feature）
    #[arg(long = "feature", value_name = "FEATURE")]
    features: Vec<String>,

    #[command(subcommand)]
    subcommand: Option<MergeCommand>,
}

#[derive(Subcommand)]
enum MergeCommand {
    /// 将 Cargo 项目结构输出到指定目录（meta/ src/ build.rs Cargo.toml）
    Output(MergeOutputArgs),
}

#[derive(Args)]
struct MergeOutputArgs {
    /// 输出目录（不存在时自动创建）
    #[arg(long, value_name = "DIR")]
    output_dir: PathBuf,
}

fn run_merge(args: MergeArgs) -> Result<()> {
    match args.subcommand {
        Some(MergeCommand::Output(out_args)) => run_merge_output(args.features, out_args),
        None => {
            let features = if args.features.is_empty() {
                vec!["default".to_string()]
            } else {
                args.features
            };

            if features.len() == 1 {
                run_single_feature_merge(&features[0])
            } else {
                run_multi_feature_merge(&features)
            }
        }
    }
}

/// 从 `MergedSpec` 和降级签名集合构建 `ApiManifest`。
fn build_api_manifest(
    feature: &str,
    spec: &merger::MergedSpec,
    degraded_sigs: &std::collections::HashSet<String>,
) -> layout::ApiManifest {
    let classes: Vec<layout::ApiClassEntry> = spec
        .class_order
        .iter()
        .map(|class_name| {
            let default_attr = format!("#[cpp(class = \"{}\")]", class_name);
            let class_attr = spec
                .class_attrs
                .get(class_name)
                .cloned()
                .unwrap_or(default_attr);
            let methods: Vec<layout::ApiMethodEntry> = spec
                .classes
                .get(class_name)
                .map(|v| v.as_slice())
                .unwrap_or(&[])
                .iter()
                .map(|m| {
                    let cpp_sig =
                        merger::extract_attr_quoted_value(&m.attr, "method = ").unwrap_or_default();
                    let is_degraded = degraded_sigs.contains(&cpp_sig);
                    layout::ApiMethodEntry {
                        cpp_sig,
                        rust_sig: m.fn_sig.clone(),
                        is_degraded,
                    }
                })
                .collect();
            layout::ApiClassEntry {
                name: class_name.clone(),
                class_attr,
                methods,
            }
        })
        .collect();

    let functions: Vec<layout::ApiFunctionEntry> = spec
        .fn_bindings
        .iter()
        .map(|fb| {
            let cpp_sig = merger::extract_attr_quoted_value(&fb.attr, "func = ").unwrap_or_default();
            let is_degraded = degraded_sigs.contains(&cpp_sig);
            layout::ApiFunctionEntry {
                cpp_sig,
                rust_sig: fb.fn_sig.clone(),
                is_degraded,
            }
        })
        .collect();

    layout::ApiManifest {
        feature: feature.to_string(),
        classes,
        functions,
    }
}

/// WalkDir 内置循环检测：遇到循环符号链接时跳过，不会无限递归。
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)
        .map_err(|e| anyhow!("create dir {}: {}", dst.display(), e))?;
    for entry in WalkDir::new(src).follow_links(true).into_iter() {
        let entry = match entry {
            Ok(e) => e,
            // 循环符号链接等可恢复错误：跳过
            Err(e) if e.loop_ancestor().is_some() => continue,
            Err(e) => return Err(anyhow!("walk {}: {}", src.display(), e)),
        };
        // 跳过源目录本身
        if entry.path() == src {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(src)
            .map_err(|e| anyhow!("strip_prefix: {}", e))?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)
                .map_err(|e| anyhow!("create dir {}: {}", target.display(), e))?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| anyhow!("create dir {}: {}", parent.display(), e))?;
            }
            std::fs::copy(entry.path(), &target).map_err(|e| {
                anyhow!(
                    "copy {} → {}: {}",
                    entry.path().display(),
                    target.display(),
                    e
                )
            })?;
        }
    }
    Ok(())
}

/// 执行 `merge output` 子命令：将 Cargo 项目结构输出到指定目录。
fn run_merge_output(features: Vec<String>, out_args: MergeOutputArgs) -> Result<()> {
    if features.len() > 1 {
        return Err(anyhow!(
            "output 子命令不支持多 feature，请只指定一个 --feature"
        ));
    }
    let feature = if features.is_empty() {
        "default".to_string()
    } else {
        features.into_iter().next().expect("features is not empty")
    };

    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);
    let lo = FeatureLayout::new(project_root.clone(), &feature);

    println!("=== cpp2rust-demo merge output ===");
    println!("项目根目录 : {}", project_root.display());
    println!("Feature    : {}", feature);
    println!("输出目录   : {}", out_args.output_dir.display());
    println!();

    if !lo.feature_root.exists() {
        return Err(anyhow!(
            "feature '{}' 不存在于 {}；请先运行 init",
            feature,
            lo.feature_root.display()
        ));
    }

    // 检查 Cargo.toml 和 build.rs 是否已生成（需要先运行 merge）
    let cargo_toml = lo.rust_dir.join("Cargo.toml");
    let build_rs = lo.rust_dir.join("build.rs");
    if !cargo_toml.exists() || !build_rs.exists() {
        return Err(anyhow!(
            "未找到 Cargo.toml 或 build.rs（位于 {}）；请先运行 'cpp2rust-demo merge --feature {}'",
            lo.rust_dir.display(),
            feature
        ));
    }

    // 确定 src 来源：优先 src.2，否则取 src（解引用 symlink）
    let src_path = {
        let src2 = lo.rust_dir.join("src.2");
        if src2.is_dir() {
            src2
        } else {
            let src_link = lo.rust_dir.join("src");
            // 若 src 是 symlink，解引用到真实路径
            if src_link.is_symlink() {
                let target = std::fs::read_link(&src_link)
                    .map_err(|e| anyhow!("read_link {}: {}", src_link.display(), e))?;
                // symlink 可能是相对路径，需拼上父目录
                if target.is_absolute() {
                    target
                } else {
                    lo.rust_dir.join(target)
                }
            } else {
                src_link
            }
        }
    };

    if !src_path.is_dir() {
        return Err(anyhow!(
            "src 目录不存在于 {}；请先运行 merge",
            src_path.display()
        ));
    }

    let out_dir = &out_args.output_dir;

    // 1. meta/ ← 复制整个 .cpp2rust/
    let cpp2rust_dir = project_root.join(".cpp2rust");
    let meta_dest = out_dir.join("meta");
    println!("复制 .cpp2rust/ → meta/ ...");
    copy_dir_all(&cpp2rust_dir, &meta_dest)?;

    // 2. src/ ← 复制 rust/src（或 src.2）内容
    let src_dest = out_dir.join("src");
    println!("复制 src → src/ ...");
    copy_dir_all(&src_path, &src_dest)?;

    // 3. build.rs
    let build_rs_dest = out_dir.join("build.rs");
    std::fs::copy(&build_rs, &build_rs_dest).map_err(|e| anyhow!("copy build.rs: {}", e))?;

    // 4. Cargo.toml
    let cargo_toml_dest = out_dir.join("Cargo.toml");
    std::fs::copy(&cargo_toml, &cargo_toml_dest)
        .map_err(|e| anyhow!("copy Cargo.toml: {}", e))?;

    println!("\n✓ cpp2rust-demo merge output 完成。");
    println!("\n输出目录结构：");
    println!("  {}/", out_dir.display());
    println!("    ├── meta/        （.cpp2rust/ 的副本）");
    println!("    ├── src/         （合并后的 Rust 源码）");
    println!("    ├── build.rs");
    println!("    └── Cargo.toml");

    Ok(())
}

fn run_single_feature_merge(feature: &str) -> Result<()> {
    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);

    println!("=== cpp2rust-demo merge ===");
    println!("项目根目录 : {}", project_root.display());
    println!("Feature    : {}", feature);
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
        "  feature '{}': {} 个单元文件，位于 {}",
        feature,
        unit_files.len(),
        canonical_src.display()
    );

    if unit_files.is_empty() {
        println!("\n未找到任何单元 .rs 文件，请先运行 'init'。");
        return Ok(());
    }

    println!("\n正在合并 {} 个单元文件...", unit_files.len());

    merger::merge_in_place(&lo.rust_dir)?;

    // ── post-merge FFI 统计 ────────────────────────────────────────────────
    let final_src = lo.rust_dir.join("src.2");
    let rust_src = if final_src.is_dir() {
        final_src
    } else {
        lo.rust_dir.join("src")
    };
    let m = collect_rust_src_metrics(&rust_src);

    // 生成 meta/merge-report.md
    let report_data = MergeReportData {
        feature,
        unit_count: unit_files.len(),
        conflicts: &[],
        rs_file_count: m.rs_files.len(),
        import_lib_files: m.import_lib_files,
        import_class_files: m.import_class_files,
        fn_binding_count: m.fn_binding_count,
        todo_count: m.todo_count,
        bad_link_name_count: m.bad_link_names.len(),
    };
    lo.save_merge_report(&report_data)?;

    // 生成 meta/api-manifest.json（C++ → Rust API 对账清单）
    let merged_spec = merger::merge_units(&unit_files);
    let degraded_sigs = merger::extract_degraded_sigs(&unit_files);
    let manifest = build_api_manifest(feature, &merged_spec, &degraded_sigs);
    lo.save_api_manifest(&manifest)?;

    println!("\n✓ cpp2rust-demo merge 完成。");
    println!("\n输出：");
    println!("  .cpp2rust/{}/", feature);
    println!("    ├── meta/");
    println!("    │   ├── merge-report.md  （merge 摘要）");
    println!("    │   └── api-manifest.json（C++ → Rust API 对账清单）");
    println!("    └── rust/");
    println!("        ├── src.1/  (init 输出备份)");
    println!("        ├── src.2/  （merge 输出，目录结构与 C++ 项目一致）");
    println!("        └── src     （符号链接 → src.2）");

    // ── §5 生成的 .rs 文件列表 ──────────────────────────────────────────────
    println!();
    println!("── 生成的 .rs 文件（共 {}，前 20 条）──", m.rs_files.len());
    for f in m.rs_files.iter().take(20) {
        // 显示相对于 rust_src 的路径（更简洁）
        let display = f
            .strip_prefix(&rust_src)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| f.display().to_string());
        println!("  {}", display);
    }
    if m.rs_files.len() > 20 {
        println!("  ...（共 {} 个文件，仅显示前 20 条）", m.rs_files.len());
    }

    // ── §6b FFI 绑定统计 ────────────────────────────────────────────────────
    println!();
    println!("── FFI 绑定统计 ──");
    println!("  import_lib!  绑定文件数：{}", m.import_lib_files);
    println!("  import_class! 绑定文件数：{}", m.import_class_files);
    println!(
        "  FFI 函数绑定总数（#[cpp(func=...)]）：{}",
        m.fn_binding_count
    );

    // link_name 一致性
    if m.bad_link_names.is_empty() {
        println!("  link_name 一致性：✓ 全部通过（无路径分隔符）");
    } else {
        println!(
            "  link_name 一致性：⚠ {} 处含路径分隔符：",
            m.bad_link_names.len()
        );
        for name in &m.bad_link_names {
            println!("    ✗ {}", name);
        }
    }

    // #include 探测
    if m.include_count > 0 {
        println!(
            "  cpp! 块 #include 指令数：{} （头文件探测已生效）",
            m.include_count
        );
    } else {
        println!("  cpp! 块 #include 指令数：0 （可能未探测到对应头文件）");
    }

    // ── §5 降级标记统计 ─────────────────────────────────────────────────────
    println!();
    if m.degraded_tags.is_empty() {
        println!("── 降级标记：✓ 无（所有特性均已完整映射）");
    } else {
        println!("── 降级标记（需人工处理，搜索 'cpp2rust-todo'）：");
        for (tag, count) in &m.degraded_tags {
            println!("  [{}] × {} 次", tag, count);
        }
    }

    // ── API 对账清单摘要 ─────────────────────────────────────────────────────
    let degraded_count = manifest
        .classes
        .iter()
        .flat_map(|c| c.methods.iter())
        .filter(|m| m.is_degraded)
        .count()
        + manifest
            .functions
            .iter()
            .filter(|f| f.is_degraded)
            .count();
    let total_methods: usize = manifest.classes.iter().map(|c| c.methods.len()).sum();
    println!();
    println!("── API 接口清单（api-manifest.json）──");
    println!("  类数量       : {}", manifest.classes.len());
    println!("  方法总数     : {}", total_methods);
    println!("  独立函数数   : {}", manifest.functions.len());
    if degraded_count == 0 {
        println!("  降级绑定数   : ✓ 无");
    } else {
        println!("  降级绑定数   : ⚠ {} 处（含 cpp2rust-todo 标记）", degraded_count);
    }

    // ── §7 汇总表 ────────────────────────────────────────────────────────────
    println!();
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│             cpp2rust-demo Merge 汇总                    │");
    println!("└─────────────────────────────────────────────────────────┘");
    println!("  feature          : {}", feature);
    println!("  合并单元文件数   : {}", unit_files.len());
    println!("  生成 .rs 文件数  : {}", m.rs_files.len());
    println!("  import_lib! 文件 : {}", m.import_lib_files);
    println!("  FFI 函数绑定数   : {}", m.fn_binding_count);
    if m.bad_link_names.is_empty() {
        println!("  link_name 检查   : ✓ 通过");
    } else {
        println!("  link_name 检查   : ⚠ {} 处异常", m.bad_link_names.len());
    }
    if m.todo_count == 0 {
        println!("  降级标记         : ✓ 无");
    } else {
        println!("  降级标记         : ⚠ {} 处（需人工完善）", m.todo_count);
    }
    println!(
        "  报告             : .cpp2rust/{}/meta/merge-report.md",
        feature
    );
    println!(
        "  API 清单         : .cpp2rust/{}/meta/api-manifest.json",
        feature
    );

    Ok(())
}

fn run_multi_feature_merge(features: &[String]) -> Result<()> {
    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);

    println!("=== cpp2rust-demo merge（多 feature）===");
    println!("项目根目录 : {}", project_root.display());
    println!("Features   : {}", features.join(", "));
    println!();

    // 验证每个 feature 存在，并确定其 canonical src 目录
    let mut feature_srcs: Vec<(&str, PathBuf)> = Vec::new();
    for feature in features {
        let lo = FeatureLayout::new(project_root.clone(), feature);
        if !lo.feature_root.exists() {
            return Err(anyhow!(
                "feature '{}' not found at {}; run init first",
                feature,
                lo.feature_root.display()
            ));
        }
        // src.1 优先（已运行过单 feature merge），否则取 src（init 直接输出）
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
        let unit_count = merger::collect_unit_rs_files(&canonical_src).len();
        println!(
            "  feature '{}': {} 个单元文件，位于 {}",
            feature,
            unit_count,
            canonical_src.display()
        );
        feature_srcs.push((feature.as_str(), canonical_src));
    }
    println!();

    // 生成合并项目到 .cpp2rust/<feat1>_<feat2>_.../rust/
    let combined_name = features.join("_");
    let combined_rust_dir = project_root
        .join(".cpp2rust")
        .join(&combined_name)
        .join("rust");
    std::fs::create_dir_all(&combined_rust_dir)
        .map_err(|e| anyhow!("create dir {}: {}", combined_rust_dir.display(), e))?;

    let feature_name_strs: Vec<&str> = features.iter().map(|s| s.as_str()).collect();

    project_generator::write_multi_feature_cargo_toml(
        &combined_rust_dir,
        &combined_name,
        &feature_name_strs,
    )?;
    project_generator::write_multi_feature_lib_rs(&combined_rust_dir, &feature_name_strs)?;
    project_generator::write_multi_feature_build_rs(&combined_rust_dir, &feature_name_strs)?;

    // 将每个 feature 的源文件复制到 src/<feature>/
    for (feature, canonical_src) in &feature_srcs {
        let feature_dest = combined_rust_dir.join("src").join(feature);
        project_generator::copy_feature_src_to_module(canonical_src, &feature_dest, feature)?;
    }

    println!("\n✓ cpp2rust-demo merge 完成。");
    println!("\n输出：");
    println!("  .cpp2rust/{}/rust/", combined_name);
    println!(
        "    ├── Cargo.toml  （package: {}；features: {}）",
        combined_name,
        features.join(", ")
    );
    println!("    ├── build.rs");
    println!("    └── src/");
    println!("        ├── lib.rs      （#[cfg(feature = \"...\")] pub mod ...;）");
    for feature in features {
        println!("        ├── {}/", feature);
    }
    println!();
    println!("单独构建某个 feature：  cargo build --features <feature>");

    Ok(())
}

fn run_init(args: InitArgs) -> Result<()> {
    let feature = &args.feature;
    let build_cmd = &args.build_cmd;

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

    // ── §6d 预处理文件行数统计 ─────────────────────────────────────────────────
    {
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

        // unit_path = C++ 编译单元对应的 Rust 模块路径（去掉首级目录，避免双重 src）
        // 例：<c_dir>/src/utils/foo.cpp.cpp2rust → "utils/foo"
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
    println!("    └── rust/       （生成的 Rust 项目：Cargo.toml、src/lib.rs、src/**/*.rs）");
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

/// 为每个编译单元生成跨模块类型引用前缀。
///
/// 当 `import_lib!` 块引用的类型在其他模块由 `import_class!` 定义时，
/// 生成对应的 `use crate::...::TypeName;` 语句。
/// 若类型未在任何模块定义（如 C typedef struct），则在本模块生成 opaque 类型声明，
/// 以便 `import_lib!` 宏展开时可以找到该类型。
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
        // fwd_decl 的格式固定为 `"class TypeName;"` ——由 extractor::build_lib_spec 的
        // `format!("class {};", name)` 生成，不含命名空间限定或 struct 前缀。
        // parse_fwd_decl 负责校验格式和标识符合法性，失败时输出警告并返回 None。
        let type_name = match parse_fwd_decl(fwd_decl, current_unit_path) {
            Some(n) => n,
            None => continue,
        };

        if local_class_names.contains(type_name) {
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
        eprintln!("错误：{:#}", e);
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
        // 未提供 --feature 时，features 为空（代码内默认为 "default"）
        assert!(merge.features.is_empty());
    }

    #[test]
    fn merge_custom_feature() {
        let args =
            Cli::try_parse_from(["cpp2rust-demo", "merge", "--feature", "core_lib"]).unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        assert_eq!(merge.features, vec!["core_lib"]);
    }

    #[test]
    fn merge_multiple_features() {
        let args = Cli::try_parse_from([
            "cpp2rust-demo",
            "merge",
            "--feature",
            "feat1",
            "--feature",
            "feat2",
            "--feature",
            "feat3",
        ])
        .unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        assert_eq!(merge.features, vec!["feat1", "feat2", "feat3"]);
    }

    #[test]
    fn merge_output_subcommand_default_feature() {
        let args = Cli::try_parse_from([
            "cpp2rust-demo",
            "merge",
            "output",
            "--output-dir",
            "/tmp/out",
        ])
        .unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        assert!(merge.features.is_empty());
        let Some(MergeCommand::Output(out)) = merge.subcommand else {
            panic!("expected Output subcommand");
        };
        assert_eq!(out.output_dir, PathBuf::from("/tmp/out"));
    }

    #[test]
    fn merge_output_subcommand_with_feature() {
        let args = Cli::try_parse_from([
            "cpp2rust-demo",
            "merge",
            "--feature",
            "mylib",
            "output",
            "--output-dir",
            "/tmp/mylib-out",
        ])
        .unwrap();
        let Commands::Merge(merge) = args.command else {
            panic!("expected Merge");
        };
        assert_eq!(merge.features, vec!["mylib"]);
        let Some(MergeCommand::Output(out)) = merge.subcommand else {
            panic!("expected Output subcommand");
        };
        assert_eq!(out.output_dir, PathBuf::from("/tmp/mylib-out"));
    }

    #[test]
    fn merge_output_requires_output_dir() {
        let result = Cli::try_parse_from(["cpp2rust-demo", "merge", "output"]);
        assert!(result.is_err());
    }

    #[test]
    fn copy_dir_all_copies_files() {
        use tempfile::TempDir;
        let src_tmp = TempDir::new().unwrap();
        let dst_tmp = TempDir::new().unwrap();

        // 建立源目录结构
        std::fs::create_dir(src_tmp.path().join("sub")).unwrap();
        std::fs::write(src_tmp.path().join("a.txt"), "hello").unwrap();
        std::fs::write(src_tmp.path().join("sub/b.txt"), "world").unwrap();

        copy_dir_all(src_tmp.path(), dst_tmp.path()).unwrap();

        assert_eq!(
            std::fs::read_to_string(dst_tmp.path().join("a.txt")).unwrap(),
            "hello"
        );
        assert_eq!(
            std::fs::read_to_string(dst_tmp.path().join("sub/b.txt")).unwrap(),
            "world"
        );
    }

    #[test]
    fn multi_feature_combined_name_uses_underscore_join() {
        // 验证多 feature 合并时目录名由 features.join("_") 生成
        let features = ["linux_x86", "arm_embedded"];
        let combined_name = features.join("_");
        assert_eq!(combined_name, "linux_x86_arm_embedded");
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

    // ── is_valid_identifier ──────────────────────────────────────────────────

    #[test]
    fn valid_identifier_simple() {
        assert!(is_valid_identifier("Foo"));
        assert!(is_valid_identifier("_bar"));
        assert!(is_valid_identifier("Vec2"));
        assert!(is_valid_identifier("my_type_123"));
    }

    #[test]
    fn invalid_identifier_empty() {
        assert!(!is_valid_identifier(""));
    }

    #[test]
    fn invalid_identifier_starts_with_digit() {
        assert!(!is_valid_identifier("1Foo"));
    }

    #[test]
    fn invalid_identifier_contains_namespace() {
        assert!(!is_valid_identifier("std::vector"));
    }

    #[test]
    fn invalid_identifier_contains_space() {
        assert!(!is_valid_identifier("Foo Bar"));
    }

    // ── build_cross_module_preamble: malformed fwd_decl ─────────────────────

    #[test]
    fn preamble_skips_malformed_fwd_decl() {
        use cpp2rust_demo::ffi_model::{FfiSpec, LibSpec};
        let spec = FfiSpec {
            lib_spec: LibSpec {
                fwd_decls: vec!["struct Foo;".to_string()], // not "class ..." format
                ..Default::default()
            },
            ..Default::default()
        };
        let map = HashMap::new();
        // malformed fwd_decl → preamble should be empty (no panic, no generated code)
        let preamble = build_cross_module_preamble(&spec, "mymod", &map);
        assert!(
            preamble.is_empty(),
            "expected empty preamble, got: {preamble:?}"
        );
    }

    #[test]
    fn preamble_skips_invalid_identifier_in_fwd_decl() {
        use cpp2rust_demo::ffi_model::{FfiSpec, LibSpec};
        let spec = FfiSpec {
            lib_spec: LibSpec {
                fwd_decls: vec!["class std::vector;".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let map = HashMap::new();
        let preamble = build_cross_module_preamble(&spec, "mymod", &map);
        assert!(
            preamble.is_empty(),
            "expected empty preamble, got: {preamble:?}"
        );
    }
}
