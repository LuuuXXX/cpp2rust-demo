//! L4 merge 集成测试
//!
//! 验证 `merger` 模块的合并、去重与备份全流程逻辑，使用真实示例文件（examples/）作为输入。
//!
//! 与 E2E 测试（需完整 C++ 工具链）不同，本文件直接操作仓库中已提交的
//! `rust_hicc/src/main.rs` 黄金文件，无需重新运行 init，因此不依赖 libclang 或 g++。

use cpp2rust_demo::merger;
use std::path::PathBuf;
use tempfile::TempDir;

// ─────────────────────────────────────────────────────────────
//  辅助函数
// ─────────────────────────────────────────────────────────────

/// 将 examples/NNN_xxx/rust_hicc/src/main.rs 中的 hicc 块提取并写入 tmp 目录，
/// 返回写入文件路径列表（可直接传给 merge_units）。
///
/// 若黄金文件不存在则返回空 Vec（测试中用 `if paths.is_empty() { return; }` 跳过）。
fn collect_golden_rs_files(examples: &[&str], tmp_dir: &std::path::Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    for name in examples {
        let golden = format!("examples/{}/rust_hicc/src/main.rs", name);
        let path = std::path::Path::new(&golden);
        if !path.exists() {
            continue;
        }
        // 为每个 example 写入独立文件，文件名取 example 名
        let dest = tmp_dir.join(format!("{}.rs", name));
        std::fs::copy(path, &dest)
            .unwrap_or_else(|e| panic!("复制黄金文件 {} 失败: {}", path.display(), e));
        if dest.exists() {
            result.push(dest);
        }
    }
    result
}

// ─────────────────────────────────────────────────────────────
//  merge_in_place: 备份与原子性 rename 验证
// ─────────────────────────────────────────────────────────────

/// 验证 merge_in_place 首次运行时正确创建 src.1 备份并将 src.2 rename 为 src。
#[test]
fn merge_in_place_creates_src1_backup_and_real_src() {
    let tmp = TempDir::new().unwrap();
    let rust_dir = tmp.path().join("rust");
    let src_dir = rust_dir.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    // 写入一个简单 .rs 文件模拟 init 输出
    std::fs::write(
        src_dir.join("foo.rs"),
        r#"hicc::cpp! { int foo() { return 1; } }
hicc::import_lib! {
    #![link_name = "foo"]
    #[cpp(func = "int foo()")]
    fn foo() -> i32;
}
"#,
    )
    .unwrap();

    merger::merge_in_place(&rust_dir).expect("merge_in_place 失败");

    // src.1 应存在（init 备份）
    assert!(
        rust_dir.join("src.1").is_dir(),
        "merge_in_place: src.1/ 备份目录应存在"
    );
    // src 应为真实目录（不是符号链接）
    let src = rust_dir.join("src");
    assert!(src.is_dir(), "merge_in_place: src/ 应为真实目录");
    assert!(!src.is_symlink(), "merge_in_place: src/ 不应为符号链接");
    // src.2 应已被 rename 为 src，因此不应存在
    assert!(
        !rust_dir.join("src.2").exists(),
        "merge_in_place: src.2 应已被 rename 为 src"
    );
}

/// 验证 merge_in_place 重复运行时 src.1 内容保持不变（不被覆盖）。
#[test]
fn merge_in_place_rerun_keeps_src1_unchanged() {
    let tmp = TempDir::new().unwrap();
    let rust_dir = tmp.path().join("rust");
    let src_dir = rust_dir.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    const UNIT_CONTENT: &str = r#"hicc::cpp! { void noop() {} }
hicc::import_lib! {
    #![link_name = "unit"]
}
"#;
    std::fs::write(src_dir.join("unit.rs"), UNIT_CONTENT).unwrap();

    // 第一次 merge
    merger::merge_in_place(&rust_dir).unwrap();
    // 读取 src.1 中文件名列表（用于后续比对）
    let src1_files: std::collections::BTreeSet<String> = std::fs::read_dir(rust_dir.join("src.1"))
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    // 第二次 merge（重复运行）
    merger::merge_in_place(&rust_dir).unwrap();

    // src.1 应仍存在且文件集合不变（内容未被重写）
    assert!(
        rust_dir.join("src.1").is_dir(),
        "merge_in_place 重复运行后 src.1 应仍存在"
    );
    let src1_files_after: std::collections::BTreeSet<String> =
        std::fs::read_dir(rust_dir.join("src.1"))
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
    assert_eq!(
        src1_files, src1_files_after,
        "merge_in_place 重复运行时 src.1 的文件集合不应改变"
    );
}

// ─────────────────────────────────────────────────────────────
//  merge_units: 去重与 API manifest 条目验证
// ─────────────────────────────────────────────────────────────

/// 验证 merge_units 能正确从多个黄金文件去重并统计类/函数绑定。
/// 使用仓库中已有的 examples/ 黄金文件，无需 libclang。
#[test]
fn merge_units_from_golden_files_deduplicates_correctly() {
    let tmp = TempDir::new().unwrap();

    // 使用若干无类依赖的简单示例
    let examples = [
        "001_hello_world",
        "002_function_overload",
        "003_default_args",
    ];

    let paths = collect_golden_rs_files(&examples, tmp.path());
    if paths.is_empty() {
        // 黄金文件不存在（如浅层 clone），跳过
        eprintln!("merge_units_from_golden_files: 黄金文件未找到，跳过");
        return;
    }

    let (spec, _) = merger::merge_units(&paths);

    // 合并后应有函数绑定（三个示例都有导出函数）
    assert!(
        !spec.fn_bindings.is_empty(),
        "merge_units: 合并 3 个示例后应有函数绑定，实际为空"
    );

    // cpp_lines 应为已去重的内容（不含重复 include 行）
    let dup_count = {
        let mut seen = std::collections::HashSet::new();
        let mut dups = 0usize;
        for line in &spec.cpp_lines {
            if !seen.insert(line.clone()) {
                dups += 1;
            }
        }
        dups
    };
    assert_eq!(dup_count, 0, "merge_units: cpp_lines 中存在重复行");
}

/// 验证 merge_units 对含类绑定的示例（如 006_class_basic）能正确提取类名和方法列表。
#[test]
fn merge_units_extracts_class_bindings() {
    let tmp = TempDir::new().unwrap();
    let paths = collect_golden_rs_files(&["006_class_basic"], tmp.path());
    if paths.is_empty() {
        eprintln!("merge_units_extracts_class_bindings: 黄金文件未找到，跳过");
        return;
    }

    let (spec, _) = merger::merge_units(&paths);

    // 006 包含 Counter 类
    assert!(
        spec.class_order.contains(&"Counter".to_string()),
        "merge_units: 006_class_basic 应含 Counter 类，实际 class_order = {:?}",
        spec.class_order
    );
    // Counter 应有至少一个方法
    let methods = spec.classes.get("Counter").map(|v| v.len()).unwrap_or(0);
    assert!(
        methods > 0,
        "merge_units: Counter 类应有至少一个方法，实际方法数 = {}",
        methods
    );
}

/// 验证 merge_units 正确收集降级签名（cpp2rust-todo 标记的绑定）。
#[test]
fn merge_units_collects_degraded_sigs_from_fn_ptr_examples() {
    let tmp = TempDir::new().unwrap();

    // 040_std_function 和 039_lambda_basic 含 FP 降级标记
    let paths = collect_golden_rs_files(&["040_std_function"], tmp.path());
    if paths.is_empty() {
        eprintln!("merge_units_collects_degraded_sigs: 黄金文件未找到，跳过");
        return;
    }

    let (spec, _) = merger::merge_units(&paths);

    assert!(
        !spec.degraded_sigs.is_empty(),
        "merge_units: 040_std_function 含 FP 降级标记，degraded_sigs 不应为空"
    );
}

// ─────────────────────────────────────────────────────────────
//  collect_unit_rs_files: 目录扫描验证
// ─────────────────────────────────────────────────────────────

/// 验证 collect_unit_rs_files 能正确扫描含子目录结构的 src/ 目录。
#[test]
fn collect_unit_rs_files_respects_subdirs() {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("src");
    std::fs::create_dir_all(src.join("sub")).unwrap();

    // 顶层文件
    std::fs::write(src.join("a.rs"), "").unwrap();
    // 子目录文件
    std::fs::write(src.join("sub").join("b.rs"), "").unwrap();
    // lib.rs 不应被收集（是聚合入口，不是单元文件）
    std::fs::write(src.join("lib.rs"), "").unwrap();

    let files = merger::collect_unit_rs_files(&src);
    let names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();

    assert!(names.contains(&"a.rs".to_string()), "应包含 a.rs");
    assert!(names.contains(&"b.rs".to_string()), "应包含 sub/b.rs");
    assert!(!names.contains(&"lib.rs".to_string()), "不应包含 lib.rs");
}
