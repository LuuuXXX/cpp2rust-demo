use crate::error::Result;
use anyhow::anyhow;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ─────────────────────────────────────────────
//  报告数据结构
// ─────────────────────────────────────────────

/// init 阶段单个编译单元的统计信息。
pub struct InitUnitStat {
    /// `.cpp2rust` 文件路径（用于显示）
    pub cpp2rust_path: String,
    /// 派生的 Rust 模块路径（如 `utils/foo`）
    pub unit_path: String,
    /// 解析到的 C++ 类数量
    pub class_count: usize,
    /// 解析到的 C++ 函数数量
    pub fn_count: usize,
    /// 解析到的 C++ 枚举数量
    pub enum_count: usize,
    /// 处理该文件耗时（毫秒）
    pub elapsed_ms: u128,
}

/// init 阶段报告所需的完整数据。
pub struct InitReportData<'a> {
    pub feature: &'a str,
    pub build_cmd: &'a str,
    pub captured_count: usize,
    pub selected_count: usize,
    pub units: &'a [InitUnitStat],
    /// 降级标签列表：`(tag, [(unit_path, count)])`，按 tag 名排序；
    /// 每个元素包含该 tag 在各编译单元中的出现次数，用于精确定位。
    pub degraded_tags: &'a [(String, Vec<(String, usize)>)],
}

/// merge 阶段报告所需的完整数据。
pub struct MergeReportData<'a> {
    pub feature: &'a str,
    pub unit_count: usize,
    /// 合并时发现的冲突描述列表
    pub conflicts: &'a [String],
    /// 合并后生成的 .rs 文件总数
    pub rs_file_count: usize,
    /// 包含 `hicc::import_lib!` 块的文件数
    pub import_lib_files: usize,
    /// 包含 `hicc::import_class!` 块的文件数
    pub import_class_files: usize,
    /// `#[cpp(func = "...")]` 绑定函数总数
    pub fn_binding_count: usize,
    /// 降级标记总数（`cpp2rust-todo`）
    pub todo_count: usize,
    /// link_name 含路径分隔符的异常数量
    pub bad_link_name_count: usize,
}

/// 从 `start` 向上逐级查找 `.cpp2rust/` 目录，返回项目根目录。
/// 若未找到则回退到 `start` 本身。
pub fn find_project_root(start: &Path) -> PathBuf {
    let mut cur = start.to_path_buf();
    loop {
        if cur.join(".cpp2rust").is_dir() {
            return cur;
        }
        match cur.parent() {
            Some(p) => cur = p.to_path_buf(),
            None => return start.to_path_buf(),
        }
    }
}

/// `.cpp2rust/<feature>/` 目录结构描述。
pub struct FeatureLayout {
    pub project_root: PathBuf,
    /// `.cpp2rust/<feature>/`
    pub feature_root: PathBuf,
    /// `.cpp2rust/<feature>/c/`
    pub c_dir: PathBuf,
    /// `.cpp2rust/<feature>/rust/`
    pub rust_dir: PathBuf,
    /// `.cpp2rust/<feature>/meta/`
    pub meta_dir: PathBuf,
}

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
            out.push_str(
                "以下特性标签需要人工处理（在生成文件中搜索 `cpp2rust-todo`）：\n\n",
            );
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
        out.push_str(
            "    │   └── merge-report.md         （由 'cpp2rust-demo merge' 生成）\n",
        );
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

        let path = self.meta_dir.join("merge-report.md");
        std::fs::write(&path, out).map_err(|e| anyhow!("write {}: {}", path.display(), e))
    }
}

/// 扫描 `.cpp2rust/<feature>/c/` 目录下所有 `*.cpp2rust` 文件。
pub fn scan_cpp2rust_files(c_dir: &Path) -> Result<Vec<PathBuf>> {
    if !c_dir.exists() {
        return Ok(vec![]);
    }
    let mut out: Vec<PathBuf> = WalkDir::new(c_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "cpp2rust"))
        .map(|e| e.path().to_path_buf())
        .collect();
    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn find_project_root_in_current_dir() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".cpp2rust")).unwrap();
        assert_eq!(find_project_root(tmp.path()), tmp.path());
    }

    #[test]
    fn find_project_root_in_parent() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".cpp2rust")).unwrap();
        let sub = tmp.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        assert_eq!(find_project_root(&sub), tmp.path());
    }

    #[test]
    fn find_project_root_fallback() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        assert_eq!(find_project_root(&sub), sub);
    }

    #[test]
    fn feature_layout_create_dirs() {
        let tmp = TempDir::new().unwrap();
        let layout = FeatureLayout::new(tmp.path().to_path_buf(), "default");
        layout.create_dirs().unwrap();
        assert!(layout.c_dir.exists());
        assert!(layout.rust_dir.exists());
        assert!(layout.meta_dir.exists());
    }

    #[test]
    fn save_build_cmd_writes_file() {
        let tmp = TempDir::new().unwrap();
        let layout = FeatureLayout::new(tmp.path().to_path_buf(), "default");
        layout.create_dirs().unwrap();
        layout
            .save_build_cmd(&["make".into(), "-j4".into()])
            .unwrap();
        let content = std::fs::read_to_string(layout.meta_dir.join("build_cmd.txt")).unwrap();
        assert_eq!(content, "make -j4");
    }

    #[test]
    fn save_selected_files_writes_json() {
        let tmp = TempDir::new().unwrap();
        let layout = FeatureLayout::new(tmp.path().to_path_buf(), "default");
        layout.create_dirs().unwrap();
        let files = vec![PathBuf::from("/foo/bar.cpp2rust")];
        layout.save_selected_files(&files).unwrap();
        let content = std::fs::read_to_string(layout.meta_dir.join("selected_files.json")).unwrap();
        assert!(content.contains("bar.cpp2rust"));
    }

    #[test]
    fn scan_cpp2rust_files_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let files = scan_cpp2rust_files(tmp.path()).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn scan_cpp2rust_files_finds_files() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("a.cpp2rust"), "").unwrap();
        std::fs::write(tmp.path().join("b.cpp"), "").unwrap();
        let files = scan_cpp2rust_files(tmp.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("a.cpp2rust"));
    }

    #[test]
    fn save_init_report_creates_file() {
        let tmp = TempDir::new().unwrap();
        let layout = FeatureLayout::new(tmp.path().to_path_buf(), "myfeature");
        layout.create_dirs().unwrap();

        let units = vec![InitUnitStat {
            cpp2rust_path: "c/src/foo.cpp.cpp2rust".into(),
            unit_path: "foo".into(),
            class_count: 2,
            fn_count: 3,
            enum_count: 1,
            elapsed_ms: 42,
        }];
        let tags = vec![("cpp_default".into(), vec![("foo".to_string(), 1usize)])];
        let data = InitReportData {
            feature: "myfeature",
            build_cmd: "make -j4",
            captured_count: 5,
            selected_count: 1,
            units: &units,
            degraded_tags: &tags,
        };
        layout.save_init_report(&data).unwrap();

        let content = std::fs::read_to_string(layout.meta_dir.join("init-report.md")).unwrap();
        assert!(content.contains("# Init 报告 — feature `myfeature`"));
        assert!(content.contains("make -j4"));
        assert!(content.contains("foo"));
        assert!(content.contains("cpp_default"));
        assert!(content.contains("2")); // class_count
    }

    #[test]
    fn save_merge_report_creates_file() {
        let tmp = TempDir::new().unwrap();
        let layout = FeatureLayout::new(tmp.path().to_path_buf(), "default");
        layout.create_dirs().unwrap();

        let data = MergeReportData {
            feature: "default",
            unit_count: 7,
            conflicts: &[],
            rs_file_count: 10,
            import_lib_files: 3,
            import_class_files: 2,
            fn_binding_count: 15,
            todo_count: 0,
            bad_link_name_count: 0,
        };
        layout.save_merge_report(&data).unwrap();

        let content = std::fs::read_to_string(layout.meta_dir.join("merge-report.md")).unwrap();
        assert!(content.contains("# Merge 报告 — feature `default`"));
        assert!(content.contains("7"));
        assert!(content.contains("*（无）*"));
        assert!(content.contains("import_lib!"));
        assert!(content.contains("✓ 全部通过"));
    }

    #[test]
    fn save_merge_report_lists_conflicts() {
        let tmp = TempDir::new().unwrap();
        let layout = FeatureLayout::new(tmp.path().to_path_buf(), "default");
        layout.create_dirs().unwrap();

        let conflicts = vec!["conflict A".into(), "conflict B".into()];
        let data = MergeReportData {
            feature: "default",
            unit_count: 2,
            conflicts: &conflicts,
            rs_file_count: 0,
            import_lib_files: 0,
            import_class_files: 0,
            fn_binding_count: 0,
            todo_count: 1,
            bad_link_name_count: 0,
        };
        layout.save_merge_report(&data).unwrap();

        let content = std::fs::read_to_string(layout.meta_dir.join("merge-report.md")).unwrap();
        assert!(content.contains("conflict A"));
        assert!(content.contains("conflict B"));
        assert!(content.contains("⚠"));
    }
}
