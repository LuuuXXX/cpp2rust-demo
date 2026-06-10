//! L4: rapidjson 完整项目端到端集成测试
//!
//! 针对 rapidjson 开源项目执行完整的 init + merge 两阶段转换，
//! 验证工具输出符合 hicc 三段式格式（hicc::cpp! / import_class! / import_lib!）。
//! 本测试作为 CI 门禁：任何破坏 rapidjson 转换输出格式的改动都会在此检测到。
//!
//! 覆盖范围：
//! - 20 个 example 文件：覆盖 rapidjson 全部公开 API（Document/Reader/Writer/
//!   PrettyWriter/Pointer/Schema/Stream 等）
//! - 10 个 unittest 文件：深度覆盖内部实现（需要 libgtest-dev，标记为 #[ignore]）
//! - 10 个 shim 文件：`references/rapidjson-refactoring/rapidjson_sys/shim/` 中的
//!   `extern "C"` 包装层，验证工具能生成完整的 import_lib! FFI 绑定

mod common;

use cpp2rust_demo::{
    ast_parser, extractor, generator::hicc_codegen, generator::project_generator,
    generator::smoke_test_gen, merger,
};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const RAPIDJSON_ROOT: &str = "references/rapidjson-refactoring/rapidjson_legacy";
const RAPIDJSON_INCLUDE: &str = "references/rapidjson-refactoring/rapidjson_legacy/include";
const RAPIDJSON_UNITTEST_DIR: &str =
    "references/rapidjson-refactoring/rapidjson_legacy/test/unittest";

/// rapidjson extern-C shim 文件目录（相对仓库根目录）
const RAPIDJSON_SHIM_DIR: &str = "references/rapidjson-refactoring/rapidjson_sys/shim";

/// shim 源文件（相对 RAPIDJSON_SHIM_DIR），每个文件对应一个 C++ 子系统的 extern-C 包装层
const SHIM_SOURCES: &[&str] = &[
    "allocator_ffi.cpp",
    "bigintegertest_ffi.cpp",
    "document_ffi.cpp",
    "encoding_ffi.cpp",
    "pointer_ffi.cpp",
    "reader_ffi.cpp",
    "schema_ffi.cpp",
    "stringbuffer_ffi.cpp",
    "value_ffi.cpp",
    "writer_ffi.cpp",
];

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

/// 处理单个 C++ 源文件：预处理 → AST → 提取 → 生成，返回 (unit_name, hicc_code)。
fn process_source(
    src_rel: &str,
    rapidjson_root: &Path,
    include_dirs: &[&str],
    preprocess_dir: &Path,
) -> Option<(String, String)> {
    let src_path = rapidjson_root.join(src_rel);
    common::process_cpp_source(&src_path, include_dirs, preprocess_dir)
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
                common::assert_valid_hicc_format(&code, &unit_name);
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
/// 使用 `cargo test -- --ignored` 显式运行此测试。
#[test]
#[ignore = "需要 libgtest-dev；运行方式：cargo test -- --ignored"]
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
                common::assert_valid_hicc_format(&code, &unit_name);
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
        let includes: &[&str] = if is_unittest {
            unittest_includes
        } else {
            example_includes
        };

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
    project_generator::write_cargo_toml(&rust_dir, "rapidjson").expect("write_cargo_toml 失败");
    project_generator::write_lib_rs(&rust_dir, &unit_paths).expect("write_lib_rs 失败");

    // ── Merge 阶段 ─────────────────────────────────────────────────
    merger::merge_in_place(&rust_dir).expect("merge_in_place 失败");

    // ── 验证输出目录结构 ────────────────────────────────────────────
    // merge_in_place 新行为：src.2 被 rename 为 src（真实目录），src.2 不再存在
    let src1 = rust_dir.join("src.1");
    let src_dir = rust_dir.join("src");

    assert!(src1.is_dir(), "merge: src.1/ 目录不存在（init 备份未生成）");
    assert!(
        src_dir.is_dir() && !src_dir.is_symlink(),
        "merge: src/ 目录不存在或为符号链接（merge 输出未生成）"
    );
    assert!(
        !rust_dir.join("src.2").is_dir() && !rust_dir.join("src.2").exists(),
        "merge: src.2 应已被 rename 为 src，不应继续存在"
    );

    // ── 验证 src/ 中的 .rs 文件内容符合 hicc 格式 ────────────────
    let merged_files = merger::collect_unit_rs_files(&src_dir);
    assert!(
        !merged_files.is_empty(),
        "merge: src/ 下未找到任何 .rs 文件"
    );

    let mut format_errors = Vec::new();
    for rs_path in &merged_files {
        let content = std::fs::read_to_string(rs_path).expect("读取合并后 .rs 文件失败");
        // lib.rs 和 mod.rs 不含 hicc 块，跳过
        let fname = rs_path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        if fname == "lib.rs" || fname == "mod.rs" {
            continue;
        }
        if let Err(e) = std::panic::catch_unwind(|| {
            common::assert_valid_hicc_format(&content, rs_path.to_str().unwrap_or("?"));
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
        "rapidjson merge: {} unit 文件 → src/ 中 {} 个 .rs 文件",
        unit_paths.len(),
        merged_files.len()
    );

    // ── cargo check：验证生成的 Rust 项目可编译 ────────────────────
    let cargo_check_output = std::process::Command::new("cargo")
        .args(["check", "--quiet"])
        .current_dir(&rust_dir)
        .output();
    match cargo_check_output {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!(
                    "cargo check 失败（init + merge 生成的 Rust 项目不可编译）:\n{}",
                    stderr
                );
            } else {
                println!("cargo check 通过");
            }
        }
        Err(e) => {
            // cargo 未安装或不可用时，跳过检查并打印警告
            println!("cargo check 跳过（cargo 不可用: {}）", e);
        }
    }
}

// ─────────────────────────────────────────────────────────────────
//  L4-Shim FFI 验证测试
// ─────────────────────────────────────────────────────────────────

/// L4-Shim-FFI：对 rapidjson_sys/shim/ 中的 extern-C 包装文件执行 init 阶段转换，
/// 验证工具能从含 `extern "C"` 函数的 C++ 文件生成完整的三段式 hicc Rust FFI 绑定。
///
/// 这是 verify-rapidjson-ffi.sh 对应的自动化回归测试。
/// shim 文件采用"不透明句柄 + extern C 包装层"模式，是为纯 C++ 库（rapidjson）
/// 生成 Rust safe FFI 的推荐工作流。
///
/// 验证要点：
/// 1. 每个 shim 文件都能成功预处理
/// 2. 每个 shim 文件生成的代码包含 `hicc::import_lib!` 块（非空 FFI 绑定）
/// 3. `import_lib!` 块包含正确的函数绑定（`#[cpp(func = ...)]`）
/// 4. link_name 不含路径分隔符
#[test]
fn rapidjson_shim_ffi_generates_importlib() {
    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("c");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    let shim_dir = PathBuf::from(RAPIDJSON_SHIM_DIR);
    let includes: &[&str] = &[RAPIDJSON_INCLUDE, RAPIDJSON_SHIM_DIR];

    assert!(
        shim_dir.exists(),
        "shim 目录不存在：{}\n  请确认 references/rapidjson-refactoring/ 子目录已就绪",
        shim_dir.display()
    );

    let mut processed = 0usize;
    let mut failed_ffi: Vec<String> = Vec::new();
    let mut skipped: Vec<&str> = Vec::new();

    for src_name in SHIM_SOURCES {
        let src_path = shim_dir.join(src_name);
        let unit_name = src_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unit")
            .to_string();

        let preprocessed =
            match common::preprocess_cpp(&src_path, includes, &preprocess_dir, &unit_name) {
                Some(p) => p,
                None => {
                    skipped.push(src_name);
                    continue;
                }
            };

        let ast = match ast_parser::parse_preprocessed(&preprocessed) {
            Ok(a) => a,
            Err(e) => {
                failed_ffi.push(format!("{}: AST 解析失败: {}", unit_name, e));
                continue;
            }
        };

        let (sys_includes, proj_header, extra_local_includes) = extractor::read_source_includes(&src_path);
        let spec = extractor::extract(&ast, &unit_name, &sys_includes, proj_header.as_deref(), &extra_local_includes);
        let code = hicc_codegen::generate(&spec);

        // 验证 1：必须有 import_lib! 块
        if !code.contains("hicc::import_lib! {") {
            failed_ffi.push(format!(
                "{}: 生成代码缺少 hicc::import_lib! 块（fn_bindings={}）",
                unit_name,
                spec.lib_spec.fn_bindings.len()
            ));
            continue;
        }

        // 验证 2：函数绑定数量 > 0
        if spec.lib_spec.fn_bindings.is_empty() {
            failed_ffi.push(format!(
                "{}: import_lib! 块存在但 fn_bindings 为空",
                unit_name
            ));
            continue;
        }

        // 验证 3：必须有 #[cpp(func = "...")] 注解
        if !code.contains("#[cpp(func = \"") {
            failed_ffi.push(format!(
                "{}: import_lib! 块缺少 #[cpp(func = \"...\")] 注解",
                unit_name
            ));
            continue;
        }

        // 验证 4：link_name 不含路径分隔符
        for line in code.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("#![link_name = \"") {
                let ln = rest.strip_suffix("\"]").unwrap_or(rest);
                if ln.contains('/') || ln.contains('\\') {
                    failed_ffi.push(format!("{}: link_name 含路径分隔符：{}", unit_name, ln));
                }
            }
        }

        println!(
            "  [OK] {}: {} fn_bindings, {} class_specs",
            unit_name,
            spec.lib_spec.fn_bindings.len(),
            spec.class_specs.len()
        );
        processed += 1;
    }

    assert!(
        skipped.is_empty(),
        "shim-ffi: {} 个文件预处理失败（g++ 是否已安装？）:\n{}",
        skipped.len(),
        skipped.join("\n")
    );

    assert!(
        failed_ffi.is_empty(),
        "shim-ffi: {} 个 shim 文件未能生成预期的 import_lib! FFI 绑定:\n{}",
        failed_ffi.len(),
        failed_ffi.join("\n")
    );

    assert_eq!(
        processed,
        SHIM_SOURCES.len(),
        "shim-ffi: 期望处理 {} 个 shim 文件，实际 {}",
        SHIM_SOURCES.len(),
        processed
    );

    println!(
        "\nshim-ffi: {} 个 shim 文件全部生成 import_lib! FFI 绑定 ✓",
        processed
    );
}

// ─────────────────────────────────────────────────────────────────
//  L4-Shim Smoke Test 生成验证
// ─────────────────────────────────────────────────────────────────

/// L4-Shim-SmokeTest：对全部 10 个 rapidjson shim 文件执行 init 流程，
/// 验证 smoke_test_gen 正确生成冒烟测试内容并写入磁盘。
///
/// 验证要点：
/// 1. 每个 shim unit 都能生成 FfiSpec（预处理 → AST → 提取）
/// 2. smoke_test_gen::generate 生成的内容含文件头注释
/// 3. 每个 shim unit 都有对应的 use ... 声明
/// 4. 每个 shim unit 都有对应的分段注释
/// 5. write_smoke_test 写入磁盘后 tests/smoke_test.rs 确实存在
/// 6. 读回文件内容与生成内容一致
/// 7. 大括号平衡检查（粗粒度语法健全性验证）
#[test]
fn rapidjson_shim_smoke_test_generated() {
    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("c");
    let rust_dir = tmp.path().join("rust");
    std::fs::create_dir_all(&preprocess_dir).unwrap();
    std::fs::create_dir_all(&rust_dir).unwrap();

    let shim_dir = PathBuf::from(RAPIDJSON_SHIM_DIR);
    let includes: &[&str] = &[RAPIDJSON_INCLUDE, RAPIDJSON_SHIM_DIR];

    assert!(
        shim_dir.exists(),
        "shim 目录不存在：{}\n  请确认 references/rapidjson-refactoring/ 子目录已就绪",
        shim_dir.display()
    );

    let mut units: Vec<(String, cpp2rust_demo::ffi_model::FfiSpec)> = Vec::new();
    let mut skipped: Vec<&str> = Vec::new();

    for src_name in SHIM_SOURCES {
        let src_path = shim_dir.join(src_name);
        // 使用与实际 init 命令一致的 derive_unit_path API（而非 file_stem()），
        // 确保测试验证的 use 路径格式与工具真实输出保持一致。
        let unit_name = project_generator::derive_unit_path(&shim_dir, &src_path);

        let preprocessed =
            match common::preprocess_cpp(&src_path, includes, &preprocess_dir, &unit_name) {
                Some(p) => p,
                None => {
                    skipped.push(src_name);
                    continue;
                }
            };

        let ast = match ast_parser::parse_preprocessed(&preprocessed) {
            Ok(a) => a,
            Err(e) => {
                panic!("shim-smoke: {} AST 解析失败: {}", unit_name, e);
            }
        };

        let (sys_includes, proj_header, extra_local_includes) = extractor::read_source_includes(&src_path);
        let spec = extractor::extract(&ast, &unit_name, &sys_includes, proj_header.as_deref(), &extra_local_includes);
        units.push((unit_name, spec));
    }

    assert!(
        skipped.is_empty(),
        "shim-smoke: {} 个 shim 文件预处理失败（g++ 是否已安装？）:\n{}",
        skipped.len(),
        skipped.join("\n")
    );

    assert_eq!(
        units.len(),
        SHIM_SOURCES.len(),
        "shim-smoke: 期望处理 {} 个 shim 文件，实际 {}",
        SHIM_SOURCES.len(),
        units.len()
    );

    // ── 生成冒烟测试内容 ───────────────────────────────────────────
    let unit_refs: Vec<(&str, &cpp2rust_demo::ffi_model::FfiSpec)> =
        units.iter().map(|(n, s)| (n.as_str(), s)).collect();
    let content = smoke_test_gen::generate(&unit_refs, "rapidjson_shim");

    // 验证 1：文件头注释
    assert!(
        content.contains("自动生成的 FFI 冒烟测试"),
        "shim-smoke: 生成内容缺少文件头注释，前 200 字符：\n{}",
        &content[..content.len().min(200)]
    );

    // 验证 2/3：每个 shim unit 都有对应的 use 声明和分段注释
    for (unit_name, _) in &units {
        assert!(
            content.contains(&format!("use rapidjson_shim::{}::*;", unit_name)),
            "shim-smoke: 缺少 unit '{}' 的 use 声明",
            unit_name
        );
        assert!(
            content.contains(&format!("// ═══ 单元：{}", unit_name)),
            "shim-smoke: 缺少 unit '{}' 的分段注释",
            unit_name
        );
    }

    // ── 写入磁盘 ───────────────────────────────────────────────────
    project_generator::write_smoke_test(&rust_dir, &content)
        .expect("shim-smoke: write_smoke_test 失败");

    // 验证 4：tests/smoke_test.rs 确实存在
    let smoke_path = rust_dir.join("tests").join("smoke_test.rs");
    assert!(
        smoke_path.exists(),
        "shim-smoke: tests/smoke_test.rs 未写入磁盘，路径：{}",
        smoke_path.display()
    );

    // 验证 5：读回内容与生成内容一致
    let written =
        std::fs::read_to_string(&smoke_path).expect("shim-smoke: 读取 smoke_test.rs 失败");
    assert_eq!(
        written, content,
        "shim-smoke: 写入磁盘的内容与生成内容不一致"
    );

    // 验证 6：大括号平衡（粗粒度语法健全性）
    let open_count = written.chars().filter(|&c| c == '{').count();
    let close_count = written.chars().filter(|&c| c == '}').count();
    assert_eq!(
        open_count, close_count,
        "shim-smoke: 生成的 smoke_test.rs 大括号不平衡（{{ {} vs }} {}），可能含语法错误",
        open_count, close_count
    );

    println!(
        "shim-smoke: {} 个 shim unit 的冒烟测试已生成，#[test] 数量：{}",
        units.len(),
        content.matches("#[test]").count()
    );
}

/// 验证生成的 rapidjson shim 冒烟测试能通过 `cargo check`。
///
/// 此测试生成完整的可编译 Rust 项目（Cargo.toml + lib.rs + unit .rs + smoke_test.rs），
/// 然后运行 `cargo check --quiet` 确认语法无误。
///
/// **跳过原因**：需要网络（下载 hicc crate 0.2）和 libclang（预处理 C++ shim 文件）。
/// CI 中由专属 job `smoke-test-cargo-check` 用 `--include-ignored` 运行。
#[test]
#[ignore = "requires libclang + network (hicc crate download)"]
fn rapidjson_shim_smoke_test_cargo_check() {
    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("c");
    let rust_dir = tmp.path().join("rust");
    std::fs::create_dir_all(&preprocess_dir).unwrap();
    std::fs::create_dir_all(&rust_dir).unwrap();

    let shim_dir = PathBuf::from(RAPIDJSON_SHIM_DIR);
    let includes: &[&str] = &[RAPIDJSON_INCLUDE, RAPIDJSON_SHIM_DIR];

    assert!(
        shim_dir.exists(),
        "shim 目录不存在：{}\n  请确认 references/rapidjson-refactoring/ 子目录已就绪",
        shim_dir.display()
    );

    // ── 处理所有 shim 文件 ──────────────────────────────────────────────
    let mut units: Vec<(String, cpp2rust_demo::ffi_model::FfiSpec)> = Vec::new();
    let mut skipped: Vec<&str> = Vec::new();

    for src_name in SHIM_SOURCES {
        let src_path = shim_dir.join(src_name);
        let unit_name = project_generator::derive_unit_path(&shim_dir, &src_path);

        let preprocessed =
            match common::preprocess_cpp(&src_path, includes, &preprocess_dir, &unit_name) {
                Some(p) => p,
                None => {
                    skipped.push(src_name);
                    continue;
                }
            };

        let ast = match ast_parser::parse_preprocessed(&preprocessed) {
            Ok(a) => a,
            Err(e) => {
                panic!("smoke-cargo-check: {} AST 解析失败: {}", unit_name, e);
            }
        };

        let (sys_includes, proj_header, extra_local_includes) = extractor::read_source_includes(&src_path);
        let spec = extractor::extract(&ast, &unit_name, &sys_includes, proj_header.as_deref(), &extra_local_includes);
        units.push((unit_name, spec));
    }

    assert!(
        skipped.is_empty(),
        "smoke-cargo-check: {} 个 shim 文件预处理失败（g++ 是否已安装？）:\n{}",
        skipped.len(),
        skipped.join("\n")
    );

    // ── 生成 Rust 项目文件 ─────────────────────────────────────────────
    let unit_refs: Vec<(&str, &cpp2rust_demo::ffi_model::FfiSpec)> =
        units.iter().map(|(n, s)| (n.as_str(), s)).collect();

    // 计算全局"已拥有"类型：在任意 unit 中含非空 ClassSpec（有方法/关联函数/析构函数）的类。
    // 这类类型由其"归属" unit 的 import_class! 块定义，其他 unit 通过 `use crate::*;` 访问，
    // 不需要重复定义。
    let globally_owned_types: std::collections::HashSet<String> = units
        .iter()
        .flat_map(|(_, spec)| {
            spec.class_specs
                .iter()
                .filter(|cs| !cs.is_empty())
                .map(|cs| cs.name.clone())
        })
        .collect();

    // 为每个没有任何 unit "拥有"（即无 ctor/dtor）但出现在函数签名中的类型，
    // 在首个引用该类型的 unit 中生成不透明的 import_class! 块。
    // 这与 init 命令中 build_cross_module_preamble 的 opaque 处理逻辑保持一致。
    let mut assigned_opaque: std::collections::HashSet<String> = std::collections::HashSet::new();

    // 1. 写 Cargo.toml
    project_generator::write_cargo_toml(&rust_dir, "rapidjson_shim")
        .expect("smoke-cargo-check: write_cargo_toml 失败");

    // 2. 写各 unit 的 .rs 文件
    for (unit_name, spec) in &units {
        // 为本 unit fwd_decls 中既不被全局拥有、也尚未在其他 unit 生成过不透明块的类型
        // 生成 repr(C) 普通结构体声明，定义不透明 Rust 类型（如 RapidJsonHandlerCallbacks）。
        // 使用普通结构体而非 hicc::import_class!，以避免 hicc ABI 变参数转换对纯 C 指针类型
        // 产生 "invalid conversion ... to void(*)(T*, ...)" 的编译错误。
        let opaque_preamble: String = spec
            .lib_spec
            .fwd_decls
            .iter()
            .filter_map(|d| {
                let type_name = d
                    .strip_prefix("class ")
                    .and_then(|s| s.strip_suffix(';'))
                    .map(str::trim)
                    .unwrap_or("");
                if type_name.is_empty()
                    || globally_owned_types.contains(type_name)
                    || assigned_opaque.contains(type_name)
                {
                    return None;
                }
                assigned_opaque.insert(type_name.to_string());
                Some(format!(
                    "#[repr(C)]\npub struct {n} {{ _private: [u8; 0] }}\n\n",
                    n = type_name
                ))
            })
            .collect();

        let code = hicc_codegen::generate(spec);
        let final_code = if opaque_preamble.is_empty() {
            code
        } else {
            format!("{}{}", opaque_preamble, code)
        };
        project_generator::write_unit_rs(&rust_dir, unit_name, &final_code)
            .unwrap_or_else(|e| panic!("smoke-cargo-check: write_unit_rs 失败 ({}): {}", unit_name, e));
    }

    // 3. 写 lib.rs（re-export 所有 unit）
    let unit_names_owned: Vec<String> = units.iter().map(|(n, _)| n.clone()).collect();
    project_generator::write_lib_rs(&rust_dir, &unit_names_owned)
        .expect("smoke-cargo-check: write_lib_rs 失败");

    // 4. 写冒烟测试
    let smoke_content = smoke_test_gen::generate(&unit_refs, "rapidjson_shim");
    project_generator::write_smoke_test(&rust_dir, &smoke_content)
        .expect("smoke-cargo-check: write_smoke_test 失败");

    // ── 执行 cargo check ───────────────────────────────────────────────
    let cargo_toml = rust_dir.join("Cargo.toml");
    assert!(
        cargo_toml.exists(),
        "smoke-cargo-check: Cargo.toml 未生成：{}",
        cargo_toml.display()
    );

    let output = std::process::Command::new("cargo")
        .args(["check", "--quiet", "--manifest-path"])
        .arg(&cargo_toml)
        .output()
        .expect("smoke-cargo-check: 无法启动 cargo check 进程");

    assert!(
        output.status.success(),
        "smoke-cargo-check: cargo check 失败（生成的冒烟测试代码存在编译错误）\n\
         stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    println!(
        "smoke-cargo-check: {} 个 shim unit 的冒烟测试 cargo check 通过 ✓",
        units.len()
    );
}

/// 回归测试：value_ffi.rs 生成代码不得包含 `class RapidJsonValueHandle;`。
///
/// 根因：`RapidJsonValueHandle` 只有工厂函数（associated_fns）和析构函数（destroy_fn），
/// 没有实例方法（methods）。若在 `import_lib!` 中生成 `class RapidJsonValueHandle;`，
/// hicc 会启用 ABI 类机制，尝试用 varargs 包装非变参函数指针，导致 C++ 编译错误。
/// 修复：`hicc_codegen` 只为有 `methods` 的类生成 `class TypeName;`。
#[test]
fn value_ffi_no_abi_class_decl_for_handle() {
    let shim_dir = PathBuf::from(RAPIDJSON_SHIM_DIR);
    let src_path = shim_dir.join("value_ffi.cpp");
    if !src_path.exists() {
        eprintln!("跳过：value_ffi.cpp 不存在");
        return;
    }
    let preprocess_dir = std::env::temp_dir().join("cpp2rust_test_vffi_regression");
    std::fs::create_dir_all(&preprocess_dir).unwrap();
    let includes: &[&str] = &[RAPIDJSON_INCLUDE, RAPIDJSON_SHIM_DIR];

    let preprocessed = match common::preprocess_cpp(&src_path, includes, &preprocess_dir, "value_ffi") {
        Some(p) => p,
        None => {
            eprintln!("跳过：value_ffi.cpp 预处理失败（g++ 是否已安装？）");
            return;
        }
    };
    let ast = ast_parser::parse_preprocessed(&preprocessed).unwrap();
    let (sys_includes, proj_header, extra_local_includes) = extractor::read_source_includes(&src_path);
    let spec = extractor::extract(&ast, "value_ffi", &sys_includes, proj_header.as_deref(), &extra_local_includes);
    let code = hicc_codegen::generate(&spec);

    assert!(
        !code.contains("class RapidJsonValueHandle;"),
        "value_ffi 生成代码不应包含 `class RapidJsonValueHandle;`（会触发错误的 hicc ABI 类机制）。\n\
         实际生成代码片段：\n{}",
        &code[..code.len().min(1000)]
    );
}

#[test]
#[ignore]
fn print_value_ffi_generated_code() {
    let shim_dir = PathBuf::from(RAPIDJSON_SHIM_DIR);
    let src_path = shim_dir.join("value_ffi.cpp");
    let preprocess_dir = std::env::temp_dir().join("cpp2rust_test_vffi");
    std::fs::create_dir_all(&preprocess_dir).unwrap();
    let includes: &[&str] = &[RAPIDJSON_INCLUDE, RAPIDJSON_SHIM_DIR];
    
    let preprocessed = common::preprocess_cpp(&src_path, includes, &preprocess_dir, "value_ffi").unwrap();
    let ast = ast_parser::parse_preprocessed(&preprocessed).unwrap();
    let (sys_includes, proj_header, extra_local_includes) = extractor::read_source_includes(&src_path);

    let interesting = ["clzll", "Pow10", "FastPath", "StrtodFast", "SkipWhitespace", "PutN"];
    println!("=== functions matching internal names (in ast.functions) ===");
    for f in &ast.functions {
        if interesting.iter().any(|&n| f.name == n) {
            println!("  name={} is_inline={} is_extern_c={} is_from_current_file={} body_offset={:?}",
                f.name, f.is_inline, f.is_extern_c, f.is_from_current_file, f.body_offset);
        }
    }
    println!("===");

    let spec = extractor::extract(&ast, "value_ffi", &sys_includes, proj_header.as_deref(), &extra_local_includes);
    
    println!("classes in ast: {:?}", ast.classes.iter().map(|c| (&c.name, c.is_from_current_file)).collect::<Vec<_>>());
    println!("class_specs: {:?}", spec.class_specs.iter().map(|cs| (&cs.name, cs.is_empty())).collect::<Vec<_>>());
    
    let code = hicc_codegen::generate(&spec);
    println!("GENERATED:\n{}", &code[..code.len().min(3000)]);
}
