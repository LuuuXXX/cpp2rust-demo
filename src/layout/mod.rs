//! 目录布局与文件操作（`.cpp2rust/<feature>/`）
//!
//! 子模块：
//! - `types` — 纯数据结构（`ApiManifest`、`FeatureLayout`、各 Entry / ReportData）
//! - `io` — 文件读写（`FeatureLayout` 的 IO 方法、`parse_smoke_test_entries`）

mod io;
pub mod types;

pub use io::parse_smoke_test_entries;
pub use types::{
    ApiClassEntry, ApiFunctionEntry, ApiManifest, ApiMethodEntry, FeatureLayout, InitReportData,
    InitUnitStat, MergeReportData, SmokeTestEntry,
};

use crate::error::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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

    #[test]
    fn save_api_manifest_creates_file() {
        use super::types::{ApiClassEntry, ApiMethodEntry};
        let tmp = TempDir::new().unwrap();
        let layout = FeatureLayout::new(tmp.path().to_path_buf(), "default");
        layout.create_dirs().unwrap();

        let manifest = ApiManifest {
            feature: "default".into(),
            classes: vec![ApiClassEntry {
                name: "Foo".into(),
                class_attr: "#[cpp(class = \"Foo\")]".into(),
                methods: vec![ApiMethodEntry {
                    cpp_sig: "int get() const".into(),
                    rust_sig: "fn get(&self) -> i32;".into(),
                    is_degraded: false,
                }],
            }],
            functions: vec![ApiFunctionEntry {
                cpp_sig: "Foo* foo_new()".into(),
                rust_sig: "fn foo_new() -> *mut Foo;".into(),
                is_degraded: false,
            }],
            template_groups: vec![],
            smoke_tests: vec![],
        };
        layout.save_api_manifest(&manifest).unwrap();

        let content = std::fs::read_to_string(layout.meta_dir.join("api-manifest.md")).unwrap();
        assert!(content.contains("# API 接口清单 — feature `default`"));
        assert!(content.contains("Foo"));
        assert!(content.contains("int get() const"));
        assert!(content.contains("fn get(&self) -> i32;"));
        assert!(content.contains("Foo* foo_new()"));
        assert!(content.contains("fn foo_new() -> *mut Foo;"));
        assert!(content.contains("✓"));
    }

    #[test]
    fn save_api_manifest_marks_degraded() {
        let tmp = TempDir::new().unwrap();
        let layout = FeatureLayout::new(tmp.path().to_path_buf(), "default");
        layout.create_dirs().unwrap();

        let manifest = ApiManifest {
            feature: "default".into(),
            classes: vec![],
            functions: vec![ApiFunctionEntry {
                cpp_sig: "void (*cb)(int)".into(),
                rust_sig: "fn cb(v: i32);".into(),
                is_degraded: true,
            }],
            template_groups: vec![],
            smoke_tests: vec![],
        };
        layout.save_api_manifest(&manifest).unwrap();

        let content = std::fs::read_to_string(layout.meta_dir.join("api-manifest.md")).unwrap();
        assert!(content.contains("void (*cb)(int)"));
        assert!(content.contains("⚠ 降级"));
    }

    #[test]
    fn save_api_manifest_renders_smoke_tests() {
        let tmp = TempDir::new().unwrap();
        let layout = FeatureLayout::new(tmp.path().to_path_buf(), "default");
        layout.create_dirs().unwrap();

        let manifest = ApiManifest {
            feature: "default".into(),
            classes: vec![],
            functions: vec![],
            template_groups: vec![],
            smoke_tests: vec![
                SmokeTestEntry {
                    fn_name: "smoke_class_basic_foo_lifecycle".into(),
                    description: "Foo 类完整生命周期".into(),
                    is_stub: false,
                },
                SmokeTestEntry {
                    fn_name: "smoke_class_basic_fn_foo_new".into(),
                    description: "含指针参数，需人工补充测试".into(),
                    is_stub: true,
                },
            ],
        };
        layout.save_api_manifest(&manifest).unwrap();

        let content = std::fs::read_to_string(layout.meta_dir.join("api-manifest.md")).unwrap();
        assert!(content.contains("## 冒烟测试"));
        assert!(content.contains("smoke_class_basic_foo_lifecycle"));
        assert!(content.contains("✓ 可运行"));
        assert!(content.contains("smoke_class_basic_fn_foo_new"));
        assert!(content.contains("⚠ 桩（需人工补充）"));
    }

    #[test]
    fn parse_smoke_test_entries_active_and_stub() {
        let content = r#"// 自动生成的 FFI 冒烟测试
use mylib::class_basic::*;

// ═══ 单元：class_basic ═══

/// 冒烟测试 A：Foo 类完整生命周期（构造 → 方法调用 → 析构）
#[test]
#[ignore = "Requires runtime environment"]
fn smoke_class_basic_foo_lifecycle() {
    let mut obj = foo_new();
    drop(obj);
}

/// 冒烟测试 B：自由函数 foo_new
#[test]
#[ignore = "Requires runtime environment"]
fn smoke_class_basic_fn_foo_create() {
    let _ = foo_create();
}

// smoke_class_basic_fn_bar_ptr: 含指针参数，需人工补充测试
// 函数签名：void* bar_ptr(void*)
// cpp2rust-todo[SMOKE]: 补充安全的入参后取消注释
// #[test]
// fn smoke_class_basic_fn_bar_ptr() { /* TODO */ }
"#;

        let entries = parse_smoke_test_entries(content);
        assert_eq!(entries.len(), 3);

        assert_eq!(entries[0].fn_name, "smoke_class_basic_foo_lifecycle");
        assert!(!entries[0].is_stub);
        assert!(entries[0].description.contains("Foo 类完整生命周期"));

        assert_eq!(entries[1].fn_name, "smoke_class_basic_fn_foo_create");
        assert!(!entries[1].is_stub);

        assert_eq!(entries[2].fn_name, "smoke_class_basic_fn_bar_ptr");
        assert!(entries[2].is_stub);
        assert!(entries[2].description.contains("含指针参数"));
    }

    #[test]
    fn parse_smoke_test_entries_deduplicates() {
        let content = r#"/// 冒烟测试 A：Foo 类
#[test]
fn smoke_foo_lifecycle() {}

/// 冒烟测试 A：Foo 类（重复）
#[test]
fn smoke_foo_lifecycle() {}
"#;
        let entries = parse_smoke_test_entries(content);
        assert_eq!(entries.len(), 1, "重复的测试函数名应去重");
    }

    #[test]
    fn parse_smoke_test_entries_empty() {
        let entries = parse_smoke_test_entries("// 仅注释，无测试");
        assert!(entries.is_empty());
    }
}
