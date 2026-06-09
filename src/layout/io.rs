//! 文件读写操作 — FeatureLayout 的 IO 方法与辅助函数
//!
//! 包含 `FeatureLayout` 的构造和所有文件读写方法（save_api_manifest、
//! save_init_report、save_merge_report 等），以及 `parse_smoke_test_entries`。

use anyhow::anyhow;
use std::path::PathBuf;

use crate::error::Result;
use super::types::{
    ApiManifest, FeatureLayout, InitReportData, MergeReportData, SmokeTestEntry,
};

impl FeatureLayout {
    pub fn new(project_root: PathBuf, feature_name: &str) -> Self {
        let feature_root = project_root.join(".cpp2rust").join(feature_name);
        Self {
            c_dir: feature_root.join("c"),
            rust_dir: feature_root.join("rust"),
            meta_dir: feature_root.join("meta"),
            feature_root,
            project_root,
        }
    }

    /// 创建所有必要的子目录。
    pub fn create_dirs(&self) -> Result<()> {
        for dir in [&self.c_dir, &self.rust_dir, &self.meta_dir] {
            std::fs::create_dir_all(dir)
                .map_err(|e| anyhow!("create dir {}: {}", dir.display(), e))?;
        }
        Ok(())
    }

    /// 写入 `meta/build_cmd.txt`。
    pub fn save_build_cmd(&self, cmd: &[String]) -> Result<()> {
        let path = self.meta_dir.join("build_cmd.txt");
        std::fs::write(&path, cmd.join(" ")).map_err(|e| anyhow!("write {}: {}", path.display(), e))
    }

    /// 写入 `meta/selected_files.json`。
    pub fn save_selected_files(&self, files: &[PathBuf]) -> Result<()> {
        let list: Vec<String> = files.iter().map(|p| p.display().to_string()).collect();
        let json = serde_json::to_string_pretty(&list)
            .map_err(|e| anyhow!("serialize selected_files: {}", e))?;
        let path = self.meta_dir.join("selected_files.json");
        std::fs::write(&path, json).map_err(|e| anyhow!("write {}: {}", path.display(), e))
    }

    /// 写入 `meta/api-manifest.md`，包含 merge 阶段生成的完整 C++ → Rust API 对账清单（Markdown 格式）。
    pub fn save_api_manifest(&self, manifest: &ApiManifest) -> Result<()> {
        let mut out = String::new();

        out.push_str(&format!(
            "# API 接口清单 — feature `{}`\n\n",
            manifest.feature
        ));
        out.push_str("由 **cpp2rust-demo merge** 生成。\n\n");
        out.push_str("本文档记录 C++ → Rust 的完整 API 绑定对账清单。✓ 表示绑定正常，⚠ 表示含降级标记（需人工处理）。\n\n");
        out.push_str("---\n\n");

        // 类绑定
        out.push_str("## 类绑定\n\n");
        if manifest.classes.is_empty() {
            out.push_str("*（无类绑定）*\n\n");
        } else {
            for class in &manifest.classes {
                out.push_str(&format!("### `{}`\n\n", class.name));
                out.push_str(&format!("**属性：** `{}`\n\n", class.class_attr));
                out.push_str("| C++ 签名 | Rust 签名 | 状态 |\n");
                out.push_str("|---------|-----------|------|\n");
                for m in &class.methods {
                    let status = if m.is_degraded { "⚠ 降级" } else { "✓" };
                    out.push_str(&format!(
                        "| `{}` | `{}` | {} |\n",
                        m.cpp_sig, m.rust_sig, status
                    ));
                }
                out.push('\n');
            }
        }
        out.push_str("---\n\n");

        // 独立函数
        out.push_str("## 独立函数\n\n");
        if manifest.functions.is_empty() {
            out.push_str("*（无独立函数绑定）*\n");
        } else {
            out.push_str("| C++ 签名 | Rust 签名 | 状态 |\n");
            out.push_str("|---------|-----------|------|\n");
            for f in &manifest.functions {
                let status = if f.is_degraded { "⚠ 降级" } else { "✓" };
                out.push_str(&format!(
                    "| `{}` | `{}` | {} |\n",
                    f.cpp_sig, f.rust_sig, status
                ));
            }
        }

        // 模板特化汇总（仅在有数据时生成）
        if !manifest.template_groups.is_empty() {
            out.push_str("\n\n---\n\n## 模板特化汇总\n\n");
            out.push_str("| 基类模板 | 已捕获特化 |\n");
            out.push_str("|---------|----------|\n");
            for (base, specs) in &manifest.template_groups {
                out.push_str(&format!("| `{}` | {} |\n", base, specs.join(", ")));
            }
        }

        // 冒烟测试清单（仅在有数据时生成）
        if !manifest.smoke_tests.is_empty() {
            out.push_str("\n\n---\n\n## 冒烟测试\n\n");
            out.push_str("以下为 `tests/smoke_test.rs` 中对应的冒烟测试函数，可用于验证 FFI 绑定正常工作。\n");
            out.push_str("运行：`cargo test -- --nocapture --include-ignored`\n\n");
            out.push_str("| 测试函数 | 说明 | 状态 |\n");
            out.push_str("|---------|------|------|\n");
            for t in &manifest.smoke_tests {
                let status = if t.is_stub { "⚠ 桩（需人工补充）" } else { "✓ 可运行" };
                out.push_str(&format!(
                    "| `{}` | {} | {} |\n",
                    t.fn_name, t.description, status
                ));
            }
        }

        let path = self.meta_dir.join("api-manifest.md");
        std::fs::write(&path, out).map_err(|e| anyhow!("write {}: {}", path.display(), e))
    }

    /// 写入 `meta/init-report.md`，包含 init 阶段的摘要报告。
    pub fn save_init_report(&self, data: &InitReportData<'_>) -> Result<()> {
        let mut out = String::new();

        out.push_str(&format!("# Init 报告 — feature `{}`\n\n", data.feature));
        out.push_str(
            "由 **cpp2rust-demo init** 生成。\n\n\
             下表中每行对应一个选定的 C++ 编译单元。\n\n---\n\n",
        );

        // 基本信息
        out.push_str("## 配置信息\n\n");
        out.push_str(&format!("- **构建命令：** `{}`\n", data.build_cmd));
        out.push_str(&format!(
            "- **捕获的 `.cpp2rust` 文件数：** {}\n",
            data.captured_count
        ));
        out.push_str(&format!("- **已选文件数：** {}\n\n", data.selected_count));
        out.push_str("---\n\n");

        // 编译单元统计表
        out.push_str("## 生成的编译单元\n\n");
        if data.units.is_empty() {
            out.push_str("*（未生成任何编译单元）*\n");
        } else {
            out.push_str("| 源文件 | Rust 模块 | 类数 | 函数数 | 枚举数 | 耗时 (ms) |\n");
            out.push_str("|--------|-----------|------|--------|--------|----------|\n");
            for u in data.units {
                out.push_str(&format!(
                    "| `{}` | `{}` | {} | {} | {} | {} |\n",
                    u.cpp2rust_path,
                    u.unit_path,
                    u.class_count,
                    u.fn_count,
                    u.enum_count,
                    u.elapsed_ms,
                ));
            }
        }
        out.push_str("\n---\n\n");

        // 降级特性
        out.push_str("## 降级特性\n\n");
        if data.degraded_tags.is_empty() {
            out.push_str("*（无 — 所有特性均已完整映射）*\n");
        } else {
            out.push_str("以下特性标签需要人工处理（在生成文件中搜索 `cpp2rust-todo`）：\n\n");
            out.push_str("| 标签 | 编译单元 | 出现次数 |\n|------|---------|----------|\n");
            for (tag, units) in data.degraded_tags {
                for (unit_path, count) in units {
                    out.push_str(&format!("| `{}` | `{}` | {} |\n", tag, unit_path, count));
                }
            }
        }
        out.push_str("\n---\n\n");

        // 输出目录结构
        out.push_str("## 输出目录结构\n\n");
        out.push_str("```\n");
        out.push_str(&format!(".cpp2rust/{}/\n", data.feature));
        out.push_str("    ├── c/          （捕获的 .cpp2rust 文件）\n");
        out.push_str("    ├── meta/\n");
        out.push_str("    │   ├── build_cmd.txt\n");
        out.push_str("    │   ├── selected_files.json\n");
        out.push_str("    │   ├── init-report.md          （本文件）\n");
        out.push_str("    │   ├── merge-report.md         （由 'cpp2rust-demo merge' 生成）\n");
        out.push_str("    │   └── api-manifest.md         （由 'cpp2rust-demo merge' 生成，C++ → Rust API 对账清单）\n");
        out.push_str(
            "    └── rust/       （生成的 Rust 项目：Cargo.toml、src/lib.rs、src/**/*.rs）\n",
        );
        out.push_str("```\n");

        let path = self.meta_dir.join("init-report.md");
        std::fs::write(&path, out).map_err(|e| anyhow!("write {}: {}", path.display(), e))
    }

    /// 写入 `meta/merge-report.md`，包含 merge 阶段的摘要报告。
    pub fn save_merge_report(&self, data: &MergeReportData<'_>) -> Result<()> {
        let mut out = String::new();

        out.push_str(&format!("# Merge 报告 — feature `{}`\n\n", data.feature));
        out.push_str(
            "由 **cpp2rust-demo merge** 生成。\n\n\
             本文档汇总了将各编译单元 init 输出合并为最终目录结构的结果。\n\n---\n\n",
        );

        // 汇总
        out.push_str("## 汇总\n\n");
        out.push_str(&format!(
            "- **已合并的编译单元文件数：** {}\n",
            data.unit_count
        ));
        out.push_str(&format!(
            "- **生成 `.rs` 文件总数：** {}\n",
            data.rs_file_count
        ));
        out.push('\n');

        // 冲突
        out.push_str("## 冲突\n\n");
        if data.conflicts.is_empty() {
            out.push_str("*（无）*\n");
        } else {
            for conflict in data.conflicts {
                out.push_str(&format!("- {}\n", conflict));
            }
        }
        out.push_str("\n---\n\n");

        // FFI 绑定统计
        out.push_str("## FFI 绑定统计\n\n");
        out.push_str("| 指标 | 数量 |\n|------|------|\n");
        out.push_str(&format!(
            "| `import_lib!` 绑定文件数 | {} |\n",
            data.import_lib_files
        ));
        out.push_str(&format!(
            "| `import_class!` 绑定文件数 | {} |\n",
            data.import_class_files
        ));
        out.push_str(&format!(
            "| FFI 函数绑定总数（`#[cpp(func=...)]`）| {} |\n",
            data.fn_binding_count
        ));
        if data.bad_link_name_count == 0 {
            out.push_str("| `link_name` 一致性 | ✓ 全部通过 |\n");
        } else {
            out.push_str(&format!(
                "| `link_name` 含路径分隔符 | ⚠ {} 处异常 |\n",
                data.bad_link_name_count
            ));
        }
        if data.todo_count == 0 {
            out.push_str("| 降级标记（`cpp2rust-todo`）| ✓ 无 |\n");
        } else {
            out.push_str(&format!(
                "| 降级标记（`cpp2rust-todo`）| ⚠ {} 处 |\n",
                data.todo_count
            ));
        }
        out.push_str("\n---\n\n");

        // 输出目录结构
        out.push_str("## 输出目录结构\n\n");
        out.push_str("```\n");
        out.push_str(&format!(".cpp2rust/{}/rust/\n", data.feature));
        out.push_str("    ├── src.1/  （init 输出备份）\n");
        out.push_str("    ├── src.2/  （merge 输出，目录结构与 C++ 项目一致）\n");
        out.push_str("    └── src     （symlink → src.2）\n");
        out.push_str("```\n");
        out.push('\n');
        out.push_str(&format!(
            "API 对账清单：`.cpp2rust/{}/meta/api-manifest.md`\n",
            data.feature
        ));

        let path = self.meta_dir.join("merge-report.md");
        std::fs::write(&path, out).map_err(|e| anyhow!("write {}: {}", path.display(), e))
    }
}

/// 从 `tests/smoke_test.rs` 文件内容中解析冒烟测试条目清单。
///
/// 支持两种形式：
/// - 有效测试：紧跟在 `/// 冒烟测试` 文档注释之后的 `fn smoke_*()` 函数
/// - 桩注释：`// smoke_*: <description>` 格式的注释行
///
/// 每个测试函数名仅输出一次（去重）。
pub fn parse_smoke_test_entries(content: &str) -> Vec<SmokeTestEntry> {
    let mut entries: Vec<SmokeTestEntry> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut pending_description: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // 文档注释：`/// 冒烟测试 A：Foo 类完整生命周期`
        if let Some(rest) = trimmed.strip_prefix("/// 冒烟测试") {
            // 保留 " A：Foo 类完整生命周期" 形式的说明（含分类标记，便于阅读）
            pending_description = Some(rest.trim().to_string());
            continue;
        }

        // 有效测试函数行：`fn smoke_<name>() {`
        if let Some(rest) = trimmed.strip_prefix("fn smoke_") {
            let fn_name_part = rest.split('(').next().unwrap_or("").trim();
            let fn_name = format!("smoke_{}", fn_name_part);
            if !seen.contains(&fn_name) {
                seen.insert(fn_name.clone());
                entries.push(SmokeTestEntry {
                    fn_name,
                    description: pending_description.take().unwrap_or_default(),
                    is_stub: false,
                });
            }
            pending_description = None;
            continue;
        }

        // 桩注释：`// smoke_<name>: <description>`
        if let Some(rest) = trimmed.strip_prefix("// smoke_") {
            if let Some((name_part, desc_part)) = rest.split_once(": ") {
                let fn_name = format!("smoke_{}", name_part.trim());
                if !seen.contains(&fn_name) {
                    seen.insert(fn_name.clone());
                    entries.push(SmokeTestEntry {
                        fn_name,
                        description: desc_part.trim().to_string(),
                        is_stub: true,
                    });
                }
            }
            pending_description = None;
            continue;
        }

        // 非文档注释行重置待处理说明，但属性行（#[…]）和空白行不重置，
        // 因为 smoke_test.rs 中文档注释后紧跟 #[test] / #[ignore = "…"] 属性，再是 fn 行
        if !trimmed.starts_with("///")
            && !trimmed.starts_with("#[")
            && !trimmed.is_empty()
        {
            pending_description = None;
        }
    }

    entries
}
