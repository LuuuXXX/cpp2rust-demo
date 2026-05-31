//! L4: rapidjson 完整项目端到端集成测试
//!
//! 针对 rapidjson 开源项目执行完整的 init + merge 两阶段转换，
//! 验证工具输出符合 hicc 三段式格式（hicc::cpp! / import_class! / import_lib!）。
//! 本测试作为 CI 门禁：任何破坏 rapidjson 转换输出格式的改动都会在此检测到。
//!
//! 覆盖范围：
//! - 20 个 example 文件：覆盖 rapidjson 全部公开 API（Document/Reader/Writer/
//!   PrettyWriter/Pointer/Schema/Stream 等）
//! - 10 个 unittest 文件：深度覆盖内部实现（Allocator/Encoding/Value/DOM 等）

use cpp2rust_demo::{ast_parser, extractor, generator::hicc_codegen, generator::project_generator, merger};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const RAPIDJSON_ROOT: &str = "references/rapidjson-refactoring/rapidjson_legacy";
const RAPIDJSON_INCLUDE: &str = "references/rapidjson-refactoring/rapidjson_legacy/include";
const RAPIDJSON_UNITTEST_DIR: &str =
    "references/rapidjson-refactoring/rapidjson_legacy/test/unittest";

/// rapidjson example 源文件（相对 RAPIDJSON_ROOT），覆盖所有公开 API
const EXAMPLE_SOURCES: &[&str] = &[
    "example/tutorial/tutorial.cpp",
    "example/simpledom/simpledom.cpp",
    "example/simplewriter/simplewriter.cpp",
    "example/simplereader/simplereader.cpp",
    "example/simplepullreader/simplepullreader.cpp",
    "example/pretty/pretty.cpp",
    "example/prettyauto/prettyauto.cpp",
    "example/condense/condense.cpp",
    "example/capitalize/capitalize.cpp",
    "example/filterkey/filterkey.cpp",
    "example/filterkeydom/filterkeydom.cpp",
    "example/sortkeys/sortkeys.cpp",
    "example/messagereader/messagereader.cpp",
    "example/serialize/serialize.cpp",
    "example/schemavalidator/schemavalidator.cpp",
    "example/parsebyparts/parsebyparts.cpp",
    "example/archiver/archiver.cpp",
    "example/jsonx/jsonx.cpp",
    "example/lookaheadparser/lookaheadparser.cpp",
    "example/traverseaspointer.cpp",
];

/// rapidjson unittest 源文件（相对 RAPIDJSON_ROOT），深度覆盖内部实现
const UNITTEST_SOURCES: &[&str] = &[
    "test/unittest/allocatorstest.cpp",
    "test/unittest/documenttest.cpp",
    "test/unittest/readertest.cpp",
    "test/unittest/writertest.cpp",
    "test/unittest/prettywritertest.cpp",
    "test/unittest/pointertest.cpp",
    "test/unittest/schematest.cpp",
    "test/unittest/valuetest.cpp",
    "test/unittest/encodingstest.cpp",
    "test/unittest/stringbuffertest.cpp",
];

// ─────────────────────────────────────────────────────────────────
//  辅助函数
// ─────────────────────────────────────────────────────────────────

/// 使用 g++ -E -C 预处理 C++ 文件，返回 .cpp2rust 文件路径。
/// 失败时返回 None（非致命错误，由调用方决定是否跳过）。
fn preprocess(
    src: &Path,
    include_dirs: &[&str],
    out_dir: &Path,
    unit_name: &str,
) -> Option<PathBuf> {
    let out = out_dir.join(format!("{}.cpp2rust", unit_name));
    let mut cmd = Command::new("g++");
    cmd.args(["-E", "-C", "-w"]);
    for inc in include_dirs {
        cmd.arg(format!("-I{}", inc));
    }
    cmd.arg(src).arg("-o").arg(&out);
    let ok = cmd.status().map(|s| s.success()).unwrap_or(false);
    if ok { Some(out) } else { None }
}

/// 验证生成的 hicc 代码符合三段式格式约束（"最初计划的内容"）：
///
/// 1. 必须包含 `hicc::cpp! {` 块（所有输出都有）
/// 2. 输出文件以 `}` 结束（最后一个宏块正确关闭）
/// 3. 每个 import_class!/import_lib! 块内部括号平衡（纯 Rust，跳过字符串）
/// 4. 若存在 `hicc::import_class!` 块，每个类必须有 `#[cpp(class` 或 `#[interface]`
/// 5. 若存在 `hicc::import_lib!` 块，必须包含 `#![link_name = "`
/// 6. 类方法绑定必须有 `#[cpp(method = "`
/// 7. 函数绑定必须有 `#[cpp(func = "`
fn assert_valid_hicc_format(code: &str, unit_name: &str) {
    // 1. 必须有 hicc::cpp! 块
    assert!(
        code.contains("hicc::cpp! {"),
        "unit '{}': 缺少 hicc::cpp! 块\n首 400 字符:\n{}",
        unit_name,
        &code[..code.len().min(400)]
    );

    // 2. 文件末尾以 } 结束（确保最后一个宏块正确关闭）
    assert!(
        code.trim_end().ends_with('}'),
        "unit '{}': 输出文件未以 }} 结束（宏块可能未正确关闭）",
        unit_name
    );

    // 3. 每个 import_class! / import_lib! 块内部括号平衡（纯 Rust 代码，可靠检查）
    for macro_prefix in &["hicc::import_class! {", "hicc::import_lib! {"] {
        let mut search = 0usize;
        while let Some(rel) = code[search..].find(macro_prefix) {
            let block_start = search + rel + macro_prefix.len();
            let mut depth = 1i32;
            let mut in_str = false;
            let mut esc = false;
            let mut closed = false;
            for c in code[block_start..].chars() {
                if esc { esc = false; continue; }
                match c {
                    '\\' if in_str => esc = true,
                    '"' => in_str = !in_str,
                    '{' if !in_str => depth += 1,
                    '}' if !in_str => {
                        depth -= 1;
                        if depth == 0 { closed = true; break; }
                    }
                    _ => {}
                }
            }
            assert!(
                closed,
                "unit '{}': {} 块未正确关闭（括号不平衡）",
                unit_name, macro_prefix
            );
            search = block_start;
        }
    }

    // 4. import_class! 块的类注解检查
    if code.contains("hicc::import_class! {") {
        assert!(
            code.contains("#[cpp(class") || code.contains("#[interface]"),
            "unit '{}': import_class! 块缺少类注解 (#[cpp(class...)] 或 #[interface])",
            unit_name
        );
    }

    // 5. import_lib! 块的 link_name 检查
    if code.contains("hicc::import_lib! {") {
        assert!(
            code.contains("#![link_name = \""),
            "unit '{}': import_lib! 块缺少 #![link_name = \"...\"]",
            unit_name
        );
    }

    // 6. 方法绑定属性检查
    if code.contains("hicc::import_class! {") && code.contains("fn ") {
        assert!(
            code.contains("#[cpp(method = \""),
            "unit '{}': import_class! 包含方法但缺少 #[cpp(method = \"...\")]",
            unit_name
        );
    }

    // 7. 函数绑定属性检查
    if code.contains("hicc::import_lib! {") && code.contains("fn ") {
        assert!(
            code.contains("#[cpp(func = \""),
            "unit '{}': import_lib! 包含函数但缺少 #[cpp(func = \"...\")]",
            unit_name
        );
    }
}

/// 处理单个 C++ 源文件：预处理 → AST → 提取 → 生成，返回 (unit_name, hicc_code)。
fn process_source(
    src_rel: &str,
    rapidjson_root: &Path,
    include_dirs: &[&str],
    preprocess_dir: &Path,
) -> Option<(String, String)> {
    let src_path = rapidjson_root.join(src_rel);
    if !src_path.exists() {
        return None;
    }

    let unit_name = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unit")
        .to_string();

    // 预处理
    let preprocessed = preprocess(&src_path, include_dirs, preprocess_dir, &unit_name)?;

    // 解析 AST
    let ast = ast_parser::parse_preprocessed(&preprocessed).ok()?;

    // 提取 FfiSpec
    let (sys_includes, proj_header) = extractor::read_source_includes(&src_path);
    let spec = extractor::extract(&ast, &unit_name, &sys_includes, proj_header.as_deref());

    // 生成 hicc 代码
    let code = hicc_codegen::generate(&spec);

    Some((unit_name, code))
}

// ─────────────────────────────────────────────────────────────────
//  L4-Init 测试
// ─────────────────────────────────────────────────────────────────

/// L4-Init-Examples：对全部 rapidjson example 文件执行 init 阶段转换。
/// 验证每个 unit 生成的 hicc 代码符合三段式格式。
#[test]
fn rapidjson_init_examples() {
    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("c");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    let root = PathBuf::from(RAPIDJSON_ROOT);
    let includes = &[RAPIDJSON_INCLUDE];

    let mut processed = 0usize;
    let mut skipped = Vec::new();

    for src_rel in EXAMPLE_SOURCES {
        match process_source(src_rel, &root, includes, &preprocess_dir) {
            Some((unit_name, code)) => {
                assert_valid_hicc_format(&code, &unit_name);
                processed += 1;
            }
            None => {
                skipped.push(*src_rel);
            }
        }
    }

    assert!(
        skipped.is_empty(),
        "init-examples: {} 个文件处理失败:\n{}",
        skipped.len(),
        skipped.join("\n")
    );
    assert_eq!(
        processed,
        EXAMPLE_SOURCES.len(),
        "init-examples: 期望处理 {} 个文件，实际 {}",
        EXAMPLE_SOURCES.len(),
        processed
    );
}

/// L4-Init-Unittests：对全部 rapidjson unittest 文件执行 init 阶段转换。
/// unittest 文件依赖 gtest，需要系统安装 libgtest-dev。
#[test]
fn rapidjson_init_unittests() {
    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("c");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    let root = PathBuf::from(RAPIDJSON_ROOT);
    let includes = &[RAPIDJSON_INCLUDE, RAPIDJSON_UNITTEST_DIR];

    let mut processed = 0usize;
    let mut skipped = Vec::new();

    for src_rel in UNITTEST_SOURCES {
        match process_source(src_rel, &root, includes, &preprocess_dir) {
            Some((unit_name, code)) => {
                assert_valid_hicc_format(&code, &unit_name);
                processed += 1;
            }
            None => {
                skipped.push(*src_rel);
            }
        }
    }

    assert!(
        skipped.is_empty(),
        "init-unittests: {} 个文件处理失败（是否已安装 libgtest-dev？）:\n{}",
        skipped.len(),
        skipped.join("\n")
    );
    assert_eq!(
        processed,
        UNITTEST_SOURCES.len(),
        "init-unittests: 期望处理 {} 个文件，实际 {}",
        UNITTEST_SOURCES.len(),
        processed
    );
}

// ─────────────────────────────────────────────────────────────────
//  L4-Merge 测试
// ─────────────────────────────────────────────────────────────────

/// L4-Merge：对 init 阶段生成的全部 unit 文件执行 merge，
/// 验证 merge 输出目录结构（src.1/、src.2/、src symlink）和合并内容格式。
#[test]
fn rapidjson_merge_phase() {
    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("c");
    let rust_dir = tmp.path().join("rust");
    std::fs::create_dir_all(&preprocess_dir).unwrap();
    std::fs::create_dir_all(&rust_dir).unwrap();

    let root = PathBuf::from(RAPIDJSON_ROOT);

    // ── Init 阶段：生成所有 unit .rs 文件 ──────────────────────────
    let all_sources: Vec<&str> = EXAMPLE_SOURCES
        .iter()
        .chain(UNITTEST_SOURCES.iter())
        .copied()
        .collect();

    let example_includes = &[RAPIDJSON_INCLUDE];
    let unittest_includes = &[RAPIDJSON_INCLUDE, RAPIDJSON_UNITTEST_DIR];

    let mut unit_paths: Vec<String> = Vec::new();

    for src_rel in &all_sources {
        let is_unittest = src_rel.contains("unittest");
        let includes: &[&str] = if is_unittest { unittest_includes } else { example_includes };

        if let Some((unit_name, code)) = process_source(src_rel, &root, includes, &preprocess_dir) {
            project_generator::write_unit_rs(&rust_dir, &unit_name, &code)
                .expect("write_unit_rs 失败");
            unit_paths.push(unit_name);
        }
    }

    assert!(
        !unit_paths.is_empty(),
        "merge: init 阶段未生成任何 unit 文件"
    );

    // 生成 Cargo.toml 与 lib.rs（merge 前必须存在 src/）
    project_generator::write_cargo_toml(&rust_dir, "rapidjson")
        .expect("write_cargo_toml 失败");
    project_generator::write_lib_rs(&rust_dir, &unit_paths)
        .expect("write_lib_rs 失败");

    // ── Merge 阶段 ─────────────────────────────────────────────────
    merger::merge_in_place(&rust_dir).expect("merge_in_place 失败");

    // ── 验证输出目录结构 ────────────────────────────────────────────
    let src1 = rust_dir.join("src.1");
    let src2 = rust_dir.join("src.2");
    let src_link = rust_dir.join("src");

    assert!(src1.is_dir(), "merge: src.1/ 目录不存在（init 备份未生成）");
    assert!(src2.is_dir(), "merge: src.2/ 目录不存在（merge 输出未生成）");
    assert!(
        src_link.is_symlink(),
        "merge: src 不是符号链接（应指向 src.2/）"
    );

    // symlink 目标必须是 src.2
    let link_target = std::fs::read_link(&src_link).expect("read_link(src) 失败");
    assert_eq!(
        link_target.to_str().unwrap_or(""),
        "src.2",
        "merge: src 符号链接目标错误，期望 src.2，实际 {}",
        link_target.display()
    );

    // ── 验证 src.2/ 中的 .rs 文件内容符合 hicc 格式 ────────────────
    let merged_files = merger::collect_unit_rs_files(&src2);
    assert!(
        !merged_files.is_empty(),
        "merge: src.2/ 下未找到任何 .rs 文件"
    );

    let mut format_errors = Vec::new();
    for rs_path in &merged_files {
        let content = std::fs::read_to_string(rs_path)
            .expect("读取合并后 .rs 文件失败");
        // lib.rs 和 mod.rs 不含 hicc 块，跳过
        let fname = rs_path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        if fname == "lib.rs" || fname == "mod.rs" {
            continue;
        }
        if let Err(e) = std::panic::catch_unwind(|| {
            assert_valid_hicc_format(&content, rs_path.to_str().unwrap_or("?"));
        }) {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                format!("{:?}", rs_path)
            };
            format_errors.push(msg);
        }
    }

    assert!(
        format_errors.is_empty(),
        "merge: {} 个文件格式验证失败:\n{}",
        format_errors.len(),
        format_errors.join("\n---\n")
    );

    // ── 统计报告 ────────────────────────────────────────────────────
    println!(
        "rapidjson merge: {} unit 文件 → src.2/ 中 {} 个 .rs 文件",
        unit_paths.len(),
        merged_files.len()
    );
}
