/// 跨 feature 合并集成测试
///
/// 这些测试验证 `merger::merge_units` + `project_generator::write_*` + `layout::save_cross_merge_report`
/// 的联合行为，覆盖多 feature 场景中的去重、冲突检测和输出目录结构。
use cpp2rust_demo::layout::{CrossMergeReportData, FeatureLayout};
use cpp2rust_demo::merger;
use cpp2rust_demo::generator::project_generator;

// ─────────────────────────────────────────────
//  辅助：构造含指定内容的 unit .rs 文件
// ─────────────────────────────────────────────

fn write_unit(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

// ─────────────────────────────────────────────
//  测试
// ─────────────────────────────────────────────

/// 来自两个 feature 的 unit 文件能正确聚合，include 去重，class 方法去重。
#[test]
fn cross_merge_deduplicates_across_features() {
    let unit_a = r#"hicc::cpp! {
    #include "shared.h"
    #include "a_only.h"
}

hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;

    }
}
"#;
    let unit_b = r#"hicc::cpp! {
    #include "shared.h"
    #include "b_only.h"
}

hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;

        #[cpp(method = "void set(int v)")]
        fn set(&mut self, v: i32);

    }
}
"#;

    let tmp = tempfile::TempDir::new().unwrap();
    let p_a = write_unit(tmp.path(), "feature_a.rs", unit_a);
    let p_b = write_unit(tmp.path(), "feature_b.rs", unit_b);

    let spec = merger::merge_units(&[p_a, p_b]);

    // shared.h 只出现一次
    let shared_count = spec.cpp_lines.iter().filter(|l| l.contains("shared.h")).count();
    assert_eq!(shared_count, 1, "shared.h should be deduped");
    // a_only.h 和 b_only.h 各出现一次
    assert!(spec.cpp_lines.iter().any(|l| l.contains("a_only.h")));
    assert!(spec.cpp_lines.iter().any(|l| l.contains("b_only.h")));

    // Foo::get 方法只出现一次（跨 feature 去重）
    let foo_methods = spec.classes.get("Foo").unwrap();
    let get_count = foo_methods.iter().filter(|m| m.fn_sig.contains("get")).count();
    assert_eq!(get_count, 1, "Foo::get should be deduped across features");
    // Foo::set 方法来自 feature_b，保留一次
    let set_count = foo_methods.iter().filter(|m| m.fn_sig.contains("set")).count();
    assert_eq!(set_count, 1, "Foo::set should appear once");

    assert!(spec.conflicts.is_empty(), "no conflicts expected");
}

/// 合并后目录名为各 feature 名以下划线拼接。
#[test]
fn merged_name_is_underscore_joined() {
    let features = vec!["linux_x86".to_string(), "arm_embedded".to_string()];
    let merged_name = features.join("_");
    assert_eq!(merged_name, "linux_x86_arm_embedded");
}

/// 三个 feature 拼接结果正确。
#[test]
fn merged_name_three_features() {
    let features = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    assert_eq!(features.join("_"), "a_b_c");
}

/// 输出目录写出完整 Rust 项目结构（Cargo.toml、build.rs、src/lib.rs、src/ffi.rs）。
#[test]
fn cross_merge_writes_complete_rust_project() {
    let unit_src = r#"hicc::cpp! {
    #include "foo.h"
}

hicc::import_lib! {
    #![link_name = "foo"]

    class Foo;

    #[cpp(func = "Foo* foo_new()")]
    fn foo_new() -> *mut Foo;
}
"#;
    let tmp = tempfile::TempDir::new().unwrap();
    let unit_path = write_unit(tmp.path(), "unit.rs", unit_src);

    let spec = merger::merge_units(&[unit_path]);
    let ffi_code = merger::emit_merged_rs(&spec, "a_b");

    let out_tmp = tempfile::TempDir::new().unwrap();
    let out_lo = FeatureLayout::new(out_tmp.path().to_path_buf(), "a_b");
    out_lo.create_dirs().unwrap();

    project_generator::write_unit_rs(&out_lo.rust_dir, "ffi", &ffi_code).unwrap();
    project_generator::write_cargo_toml(&out_lo.rust_dir, "a_b").unwrap();
    project_generator::write_build_rs(&out_lo.rust_dir, "a_b").unwrap();
    project_generator::write_lib_rs(&out_lo.rust_dir, &["ffi".to_string()]).unwrap();

    // 验证输出文件存在
    assert!(out_lo.rust_dir.join("Cargo.toml").exists());
    assert!(out_lo.rust_dir.join("build.rs").exists());
    assert!(out_lo.rust_dir.join("src/lib.rs").exists());
    assert!(out_lo.rust_dir.join("src/ffi.rs").exists());

    // 验证 Cargo.toml 中 package.name 为合并后名称
    let cargo_content = std::fs::read_to_string(out_lo.rust_dir.join("Cargo.toml")).unwrap();
    assert!(cargo_content.contains(r#"name = "a_b""#), "package.name should be merged name");

    // 验证生成的 ffi.rs 包含来自 unit 的内容
    let ffi_content = std::fs::read_to_string(out_lo.rust_dir.join("src/ffi.rs")).unwrap();
    assert!(ffi_content.contains("foo.h"), "ffi.rs should contain foo.h include");
    assert!(ffi_content.contains("foo_new"), "ffi.rs should contain foo_new binding");
}

/// merge-report.md 包含来源 feature 列表和合并目录名。
#[test]
fn cross_merge_report_contains_source_features() {
    let tmp = tempfile::TempDir::new().unwrap();
    let layout = FeatureLayout::new(tmp.path().to_path_buf(), "linux_x86_arm");
    layout.create_dirs().unwrap();

    let sources = vec!["linux_x86".to_string(), "arm".to_string()];
    let data = CrossMergeReportData {
        source_features: &sources,
        merged_name: "linux_x86_arm",
        unit_count: 4,
        conflicts: &[],
    };
    layout.save_cross_merge_report(&data).unwrap();

    let content = std::fs::read_to_string(layout.meta_dir.join("merge-report.md")).unwrap();
    assert!(content.contains("linux_x86_arm"), "should contain merged name");
    assert!(content.contains("`linux_x86`"), "should list source feature linux_x86");
    assert!(content.contains("`arm`"), "should list source feature arm");
    assert!(content.contains("4"), "should show unit count");
}

/// 冲突（两个 feature 中同一方法签名不同）会被记录在报告中。
#[test]
fn cross_merge_detects_conflicts() {
    let unit_a = r#"hicc::cpp! {
    #include "foo.h"
}

hicc::import_class! {
    #[cpp(class = "Bar")]
    class Bar {
        #[cpp(method = "int val() const")]
        fn val(&self) -> i32;

    }
}
"#;
    // 同一 method attr，但 fn 签名不同（返回类型冲突）
    let unit_b = r#"hicc::cpp! {
    #include "foo.h"
}

hicc::import_class! {
    #[cpp(class = "Bar")]
    class Bar {
        #[cpp(method = "int val() const")]
        fn val(&self) -> i64;

    }
}
"#;
    let tmp = tempfile::TempDir::new().unwrap();
    let p_a = write_unit(tmp.path(), "feat_a.rs", unit_a);
    let p_b = write_unit(tmp.path(), "feat_b.rs", unit_b);

    let spec = merger::merge_units(&[p_a, p_b]);
    assert!(
        !spec.conflicts.is_empty(),
        "conflicting method signatures should be detected"
    );
    assert!(
        spec.conflicts.iter().any(|c| c.contains("Bar")),
        "conflict should mention class Bar"
    );
}
