//! `merge` 子命令实现
//!
//! 将各编译单元生成的 `.rs` 文件合并为模块级输出，支持单/多 feature 模式及 --output-dir 导出。

use anyhow::anyhow;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::generator::project_generator;
use crate::layout::{self, ApiManifest, FeatureLayout, MergeReportData};
use crate::merger;
use crate::metrics::collect_rust_src_metrics;

/// 执行 `merge` 命令：合并单/多 feature，并可选导出到 `--output-dir`。
pub fn run_merge(features: Vec<String>, output_dir: Option<PathBuf>) -> Result<()> {
    let features = if features.is_empty() {
        vec!["default".to_string()]
    } else {
        features
    };

    // 在顶层获取一次，传入子函数，避免重复调用
    let cwd = std::env::current_dir().map_err(|e| anyhow!("current_dir: {}", e))?;
    let project_root = layout::find_project_root(&cwd);

    // 先执行普通 merge（单/多 feature）
    let rust_dir = if features.len() == 1 {
        run_single_feature_merge(&features[0], &project_root)?
    } else {
        run_multi_feature_merge(&features, &project_root)?
    };

    // 若指定了 --output-dir，merge 完成后追加导出步骤
    if let Some(out_dir) = output_dir {
        let merged_feature_name = features.join("_");
        run_merge_output(&rust_dir, &out_dir, &merged_feature_name, &project_root)?;
    }

    Ok(())
}

/// 执行 `merge --output-dir` 后处理：将 merge 生成的 Cargo 项目结构导出到指定目录。
///
/// 此函数始终在普通 merge 完成后调用，因此 `rust_dir/src` 保证是真实目录。
fn run_merge_output(
    rust_dir: &Path,
    out_dir: &Path,
    merged_feature_name: &str,
    project_root: &Path,
) -> Result<()> {
    println!("\n=== cpp2rust-demo merge --output-dir ===");
    println!("Feature    : {}", merged_feature_name);
    println!("输出目录   : {}", out_dir.display());
    println!();

    let src_path = rust_dir.join("src");

    if !src_path.is_dir() {
        return Err(anyhow!(
            "src 目录不存在于 {}；merge 阶段可能未成功",
            src_path.display()
        ));
    }

    // 1. meta/ ← 复制整个 .cpp2rust/
    let cpp2rust_dir = project_root.join(".cpp2rust");
    let meta_dest = out_dir.join("meta");
    println!("复制 .cpp2rust/ → meta/ ...");
    merger::copy_dir_all(&cpp2rust_dir, &meta_dest)?;

    // 2. src/ ← 复制 rust/src 内容
    let src_dest = out_dir.join("src");
    println!("复制 src → src/ ...");
    merger::copy_dir_all(&src_path, &src_dest)?;

    // 3. build.rs
    let build_rs = rust_dir.join("build.rs");
    if build_rs.exists() {
        let build_rs_dest = out_dir.join("build.rs");
        std::fs::copy(&build_rs, &build_rs_dest).map_err(|e| anyhow!("copy build.rs: {}", e))?;
    }

    // 4. Cargo.toml
    let cargo_toml = rust_dir.join("Cargo.toml");
    if cargo_toml.exists() {
        let cargo_toml_dest = out_dir.join("Cargo.toml");
        std::fs::copy(&cargo_toml, &cargo_toml_dest)
            .map_err(|e| anyhow!("copy Cargo.toml: {}", e))?;
    }

    println!("\n✓ cpp2rust-demo merge --output-dir 完成。");
    println!("\n输出目录结构：");
    println!("  {}/", out_dir.display());
    println!("    ├── meta/        （.cpp2rust/ 的副本）");
    println!("    ├── src/         （合并后的 Rust 源码）");
    println!("    ├── build.rs");
    println!("    └── Cargo.toml");

    Ok(())
}

/// 执行单 feature merge。
pub fn run_single_feature_merge(feature: &str, project_root: &std::path::Path) -> Result<PathBuf> {
    println!("=== cpp2rust-demo merge ===");
    println!("项目根目录 : {}", project_root.display());
    println!("Feature    : {}", feature);
    println!();

    let lo = FeatureLayout::new(project_root.to_path_buf(), feature);
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
        return Ok(lo.rust_dir);
    }

    println!("\n正在合并 {} 个单元文件...", unit_files.len());

    merger::merge_in_place(&lo.rust_dir)?;

    // ── post-merge FFI 统计 ────────────────────────────────────────────────
    // merge_in_place 完成后，src.2 已原子性 rename 为 src，此处直接使用 src。
    let rust_src = lo.rust_dir.join("src");
    let m = collect_rust_src_metrics(&rust_src);

    // 生成 meta/api-manifest.md（C++ → Rust API 对账清单）
    let merged_spec = merger::merge_units(&unit_files);

    // 生成 meta/merge-report.md
    // 注：merge_units 需在 save_merge_report 之前调用，以便将冲突信息写入报告
    let report_data = MergeReportData {
        feature,
        unit_count: unit_files.len(),
        conflicts: &merged_spec.conflicts,
        rs_file_count: m.rs_files.len(),
        import_lib_files: m.import_lib_files,
        import_class_files: m.import_class_files,
        fn_binding_count: m.fn_binding_count,
        todo_count: m.todo_count,
        bad_link_name_count: m.bad_link_names.len(),
    };
    lo.save_merge_report(&report_data)?;

    let manifest = build_api_manifest(feature, &merged_spec, &merged_spec.degraded_sigs);
    lo.save_api_manifest(&manifest)?;

    println!("\n✓ cpp2rust-demo merge 完成。");
    println!("\n输出：");
    println!("  .cpp2rust/{}/", feature);
    println!("    ├── meta/");
    println!("    │   ├── merge-report.md  （merge 摘要）");
    println!("    │   └── api-manifest.md  （C++ → Rust API 对账清单）");
    println!("    └── rust/");
    println!("        ├── src.1/  （init 输出备份，首次运行时 rename from src）");
    println!("        └── src/    （merge 输出，真实目录，与 C++ 项目目录结构一致）");

    print_ffi_stats(&m, &rust_src);
    print_manifest_summary(&manifest);
    print_merge_summary_table(feature, unit_files.len(), &m);

    Ok(lo.rust_dir)
}

/// 执行多 feature merge。
fn run_multi_feature_merge(features: &[String], project_root: &std::path::Path) -> Result<PathBuf> {
    println!("=== cpp2rust-demo merge（多 feature）===");
    println!("项目根目录 : {}", project_root.display());
    println!("Features   : {}", features.join(", "));
    println!();

    // 验证每个 feature 存在，并确定其 canonical src 目录
    let mut feature_srcs: Vec<(&str, PathBuf)> = Vec::new();
    for feature in features {
        let lo = FeatureLayout::new(project_root.to_path_buf(), feature);
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

    Ok(combined_rust_dir)
}

/// 从 `MergedSpec` 和降级签名集合构建 `ApiManifest`。
fn build_api_manifest(
    feature: &str,
    spec: &merger::MergedSpec,
    degraded_sigs: &std::collections::HashSet<String>,
) -> ApiManifest {
    use crate::layout::{ApiClassEntry, ApiFunctionEntry, ApiMethodEntry};

    let classes: Vec<ApiClassEntry> = spec
        .class_order
        .iter()
        .map(|class_name| {
            let default_attr = format!("#[cpp(class = \"{}\")]", class_name);
            let class_attr = spec
                .class_attrs
                .get(class_name)
                .cloned()
                .unwrap_or(default_attr);
            let methods: Vec<ApiMethodEntry> = spec
                .classes
                .get(class_name)
                .map(|v| v.as_slice())
                .unwrap_or(&[])
                .iter()
                .map(|m| {
                    let cpp_sig =
                        merger::extract_attr_quoted_value(&m.attr, "method = ").unwrap_or_default();
                    let is_degraded = degraded_sigs.contains(&cpp_sig);
                    ApiMethodEntry {
                        cpp_sig,
                        rust_sig: m.fn_sig.clone(),
                        is_degraded,
                    }
                })
                .collect();
            ApiClassEntry {
                name: class_name.clone(),
                class_attr,
                methods,
            }
        })
        .collect();

    let functions: Vec<ApiFunctionEntry> = spec
        .fn_bindings
        .iter()
        .map(|fb| {
            let cpp_sig =
                merger::extract_attr_quoted_value(&fb.attr, "func = ").unwrap_or_default();
            let is_degraded = degraded_sigs.contains(&cpp_sig);
            ApiFunctionEntry {
                cpp_sig,
                rust_sig: fb.fn_sig.clone(),
                is_degraded,
            }
        })
        .collect();

    // 模板特化分组：HashMap → 排序后的 Vec（保证报告输出稳定）
    let mut template_groups: Vec<(String, Vec<String>)> = spec
        .template_groups
        .iter()
        .map(|(base, specs)| (base.clone(), specs.clone()))
        .collect();
    template_groups.sort_by(|a, b| a.0.cmp(&b.0));

    ApiManifest {
        feature: feature.to_string(),
        classes,
        functions,
        template_groups,
    }
}

// ─── 打印辅助函数 ─────────────────────────────────────────────────────────────

/// 打印 merge 后生成的 `.rs` 文件列表及 FFI 绑定统计，包含降级标记汇总。
fn print_ffi_stats(m: &crate::metrics::RustSrcMetrics, rust_src: &Path) {
    // ── 生成的 .rs 文件列表 ──────────────────────────────────────────────
    println!();
    println!("── 生成的 .rs 文件（共 {}，前 20 条）──", m.rs_files.len());
    for f in m.rs_files.iter().take(20) {
        let display = f
            .strip_prefix(rust_src)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| f.display().to_string());
        println!("  {}", display);
    }
    if m.rs_files.len() > 20 {
        println!("  ...（共 {} 个文件，仅显示前 20 条）", m.rs_files.len());
    }

    // ── FFI 绑定统计 ────────────────────────────────────────────────────
    println!();
    println!("── FFI 绑定统计 ──");
    println!("  import_lib!  绑定文件数：{}", m.import_lib_files);
    println!("  import_class! 绑定文件数：{}", m.import_class_files);
    println!(
        "  FFI 函数绑定总数（#[cpp(func=...)]）：{}",
        m.fn_binding_count
    );

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

    if m.include_count > 0 {
        println!(
            "  cpp! 块 #include 指令数：{} （头文件探测已生效）",
            m.include_count
        );
    } else {
        println!("  cpp! 块 #include 指令数：0 （可能未探测到对应头文件）");
    }

    // ── 降级标记统计 ─────────────────────────────────────────────────────
    println!();
    if m.degraded_tags.is_empty() {
        println!("── 降级标记：✓ 无（所有特性均已完整映射）");
    } else {
        println!("── 降级标记（需人工处理，搜索 'cpp2rust-todo'）：");
        for (tag, count) in &m.degraded_tags {
            println!("  [{}] × {} 次", tag, count);
        }
    }
}

/// 打印 API 接口清单摘要（类数、方法数、降级绑定数）。
fn print_manifest_summary(manifest: &ApiManifest) {
    let degraded_count = manifest
        .classes
        .iter()
        .flat_map(|c| c.methods.iter())
        .filter(|m| m.is_degraded)
        .count()
        + manifest.functions.iter().filter(|f| f.is_degraded).count();
    let total_methods: usize = manifest.classes.iter().map(|c| c.methods.len()).sum();
    println!();
    println!("── API 接口清单（api-manifest.md）──");
    println!("  类数量       : {}", manifest.classes.len());
    println!("  方法总数     : {}", total_methods);
    println!("  独立函数数   : {}", manifest.functions.len());
    if degraded_count == 0 {
        println!("  降级绑定数   : ✓ 无");
    } else {
        println!(
            "  降级绑定数   : ⚠ {} 处（含 cpp2rust-todo 标记）",
            degraded_count
        );
    }
}

/// 打印 merge 完成后的汇总表格。
fn print_merge_summary_table(
    feature: &str,
    unit_count: usize,
    m: &crate::metrics::RustSrcMetrics,
) {
    println!();
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│             cpp2rust-demo Merge 汇总                    │");
    println!("└─────────────────────────────────────────────────────────┘");
    println!("  feature          : {}", feature);
    println!("  合并单元文件数   : {}", unit_count);
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
        "  API 清单         : .cpp2rust/{}/meta/api-manifest.md",
        feature
    );
}
