//! 文件读写操作 — FeatureLayout 的 IO 方法与辅助函数
//!
//! 包含 `FeatureLayout` 的构造和所有文件读写方法（save_api_manifest、
//! save_init_report、save_merge_report 等），以及 `parse_smoke_test_entries`。

use anyhow::anyhow;
use std::path::PathBuf;

use crate::error::Result;
use super::types::{
    ApiManifest, FeatureLayout, InitReportData, MergeReportData,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::types::{InitUnitStat, MergeReportData};
    use tempfile::TempDir;

    fn make_layout(tmp: &TempDir) -> FeatureLayout {
        let root = tmp.path().to_path_buf();
        let feature_root = root.join(".cpp2rust").join("test");
        let meta_dir = feature_root.join("meta");
        std::fs::create_dir_all(&meta_dir).unwrap();
        FeatureLayout {
            project_root: root.clone(),
            feature_root: feature_root.clone(),
            c_dir: feature_root.join("c"),
            rust_dir: feature_root.join("rust"),
            meta_dir,
        }
    }

    // ── save_init_report ─────────────────────────────────────────────────────

    #[test]
    fn save_init_report_empty_units() {
        let tmp = TempDir::new().unwrap();
        let lo = make_layout(&tmp);
        let data = InitReportData {
            feature: "test",
            build_cmd: "make",
            captured_count: 0,
            selected_count: 0,
            units: &[],
            degraded_tags: &[],
        };
        lo.save_init_report(&data).unwrap();
        let content = std::fs::read_to_string(lo.meta_dir.join("init-report.md")).unwrap();
        assert!(content.contains("feature `test`"));
        assert!(content.contains("未生成任何编译单元"));
        assert!(content.contains("无 — 所有特性均已完整映射"));
    }

    #[test]
    fn save_init_report_with_units_and_tags() {
        let tmp = TempDir::new().unwrap();
        let lo = make_layout(&tmp);
        let units = vec![InitUnitStat {
            cpp2rust_path: "c/foo.cpp.cpp2rust".to_string(),
            unit_path: "foo".to_string(),
            class_count: 1,
            fn_count: 2,
            enum_count: 0,
            elapsed_ms: 42,
        }];
        let tags = vec![("FP".to_string(), vec![("foo".to_string(), 3usize)])];
        let data = InitReportData {
            feature: "myfeature",
            build_cmd: "cmake --build .",
            captured_count: 1,
            selected_count: 1,
            units: &units,
            degraded_tags: &tags,
        };
        lo.save_init_report(&data).unwrap();
        let content = std::fs::read_to_string(lo.meta_dir.join("init-report.md")).unwrap();
        assert!(content.contains("myfeature"));
        assert!(content.contains("foo.cpp.cpp2rust"));
        assert!(content.contains("FP"));
        assert!(content.contains("3"));
    }

    // ── save_merge_report ────────────────────────────────────────────────────

    #[test]
    fn save_merge_report_no_conflicts() {
        let tmp = TempDir::new().unwrap();
        let lo = make_layout(&tmp);
        let data = MergeReportData {
            feature: "default",
            unit_count: 5,
            conflicts: &[],
            rs_file_count: 3,
            import_lib_files: 2,
            import_class_files: 1,
            fn_binding_count: 10,
            todo_count: 0,
            bad_link_name_count: 0,
        };
        lo.save_merge_report(&data).unwrap();
        let content = std::fs::read_to_string(lo.meta_dir.join("merge-report.md")).unwrap();
        assert!(content.contains("default"));
        assert!(content.contains("无"));
        assert!(content.contains("✓ 全部通过"));
    }

    #[test]
    fn save_merge_report_with_conflicts_and_todos() {
        let tmp = TempDir::new().unwrap();
        let lo = make_layout(&tmp);
        let conflicts = vec!["method Foo::bar 在两个翻译单元中定义不一致".to_string()];
        let data = MergeReportData {
            feature: "multi",
            unit_count: 3,
            conflicts: &conflicts,
            rs_file_count: 4,
            import_lib_files: 1,
            import_class_files: 2,
            fn_binding_count: 7,
            todo_count: 2,
            bad_link_name_count: 1,
        };
        lo.save_merge_report(&data).unwrap();
        let content = std::fs::read_to_string(lo.meta_dir.join("merge-report.md")).unwrap();
        assert!(content.contains("Foo::bar"));
        assert!(content.contains("⚠ 2 处"));
        assert!(content.contains("⚠ 1 处异常"));
    }
}
