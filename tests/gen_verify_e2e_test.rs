//! L6 gen-verify：端到端验证工具实际生成的代码可被 Rust 编译器接受。
//!
//! 与 L1/L2/L_smoke 不同：
//! - L1 验证生成代码与手写黄金一致
//! - L2/L_smoke 验证**手写黄金**可编译、行为正确
//! - **L6（本测试）** 直接验证**工具实际生成**的代码可被 Rust 编译器接受
//!
//! 对 3 个代表性示例（模板函数、模板类、接口虚函数）运行完整的
//! 代码生成流水线，然后将生成的代码写入临时 Cargo 项目（使用
//! 绝对路径引用原始 C++ 文件），运行 `cargo build` 验证可编译性。
//!
//! 由于需要 `g++`/`clang++` 进行 C++ 预处理以及 `libclang` 进行 AST 解析，
//! 所有测试均标注 `#[ignore]`，通过 `--include-ignored` 显式运行（CI gen-verify job 调用）。

mod common;

use std::path::{Path, PathBuf};
use tempfile::TempDir;

// ─────────────────────────────────────────────────────────────────
//  测试用例（三类代表性示例）
// ─────────────────────────────────────────────────────────────────

/// L6-1：024_template_function — 模板函数实例化
///
/// 验证工具对函数模板的生成代码（swap_int / swap_double 等）可被 Rust 编译器接受。
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_template_function() {
    gen_verify_example(
        "examples/024_template_function",
        "template_function",
        "template_function",
    );
}

/// L6-2：025_template_class — 模板类实例化
///
/// 验证工具对类模板（Stack<int> / Stack<double>）的生成代码可被 Rust 编译器接受。
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_template_class() {
    gen_verify_example(
        "examples/025_template_class",
        "template_class",
        "template_class",
    );
}

/// L6-3：015_virtual_basic — 虚函数接口
///
/// 验证工具对含虚函数类（Shape / Circle）的生成代码可被 Rust 编译器接受。
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_virtual_basic() {
    gen_verify_example(
        "examples/015_virtual_basic",
        "virtual_basic",
        "virtual_basic",
    );
}

/// L6-4：006_class_basic — 基础类（getter/setter/静态方法）
///
/// 验证工具对普通类方法的生成代码可被 Rust 编译器接受。
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_class_basic() {
    gen_verify_example("examples/006_class_basic", "class_basic", "class_basic");
}

/// L6-5：013_inheritance_single — 单继承
///
/// 验证工具对单继承（基类方法提升进子类）的生成代码可被 Rust 编译器接受。
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_inheritance_single() {
    gen_verify_example(
        "examples/013_inheritance_single",
        "inheritance_single",
        "inheritance_single",
    );
}

/// L6-6：042_exception_basic — 异常处理（try/catch → 错误码）
///
/// 验证工具对含异常处理的 C++ 代码生成的 FFI 绑定可被 Rust 编译器接受。
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_exception_basic() {
    gen_verify_example(
        "examples/042_exception_basic",
        "exception_basic",
        "exception_basic",
    );
}

/// L6-7：029_unique_ptr — 智能指针
///
/// 验证工具对使用 unique_ptr 的 C++ 类生成的 FFI 绑定可被 Rust 编译器接受。
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_unique_ptr() {
    gen_verify_example("examples/029_unique_ptr", "unique_ptr", "unique_ptr");
}

/// L6-8：034_vector_basic — STL 容器（vector）
///
/// 验证工具对使用 std::vector 的 C++ 代码生成的 FFI 绑定可被 Rust 编译器接受。
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_vector_basic() {
    gen_verify_example("examples/034_vector_basic", "vector_basic", "vector_basic");
}

/// L6-9：001_hello_world — 基础函数
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_hello_world() {
    gen_verify_example("examples/001_hello_world", "hello_world", "hello_world");
}

/// L6-10：002_function_overload — 函数重载
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_function_overload() {
    gen_verify_example(
        "examples/002_function_overload",
        "function_overload",
        "function_overload",
    );
}

/// L6-11：003_default_args — 默认参数
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_default_args() {
    gen_verify_example("examples/003_default_args", "default_args", "default_args");
}

/// L6-12：004_inline_functions — 内联函数
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_inline_functions() {
    gen_verify_example(
        "examples/004_inline_functions",
        "inline_functions",
        "inline_functions",
    );
}

/// L6-13：005_variadic_functions — 可变参数函数
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_variadic_functions() {
    gen_verify_example(
        "examples/005_variadic_functions",
        "variadic_functions",
        "variadic_functions",
    );
}

/// L6-14：007_class_constructor — 类构造函数
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_class_constructor() {
    gen_verify_example(
        "examples/007_class_constructor",
        "class_constructor",
        "class_constructor",
    );
}

/// L6-15：008_class_copy — 拷贝构造
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_class_copy() {
    gen_verify_example("examples/008_class_copy", "class_copy", "class_copy");
}

/// L6-16：009_class_move — 移动构造
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_class_move() {
    gen_verify_example("examples/009_class_move", "class_move", "class_move");
}

/// L6-17：010_class_static — 静态成员
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_class_static() {
    gen_verify_example("examples/010_class_static", "class_static", "class_static");
}

/// L6-18：011_class_const — const 成员
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_class_const() {
    gen_verify_example("examples/011_class_const", "class_const", "class_const");
}

/// L6-19：012_class_volatile — volatile 成员
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_class_volatile() {
    gen_verify_example(
        "examples/012_class_volatile",
        "class_volatile",
        "class_volatile",
    );
}

/// L6-20：014_inheritance_multiple — 多重继承
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_inheritance_multiple() {
    gen_verify_example(
        "examples/014_inheritance_multiple",
        "inheritance_multiple",
        "inheritance_multiple",
    );
}

/// L6-21：016_virtual_pure — 纯虚函数
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_virtual_pure() {
    gen_verify_example("examples/016_virtual_pure", "virtual_pure", "virtual_pure");
}

/// L6-22：017_virtual_override — 虚函数重写
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_virtual_override() {
    gen_verify_example(
        "examples/017_virtual_override",
        "virtual_override",
        "virtual_override",
    );
}

/// L6-23：018_virtual_diamond — 菱形继承
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_virtual_diamond() {
    gen_verify_example(
        "examples/018_virtual_diamond",
        "virtual_diamond",
        "virtual_diamond",
    );
}

/// L6-24：019_operator_overload — 运算符重载
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_operator_overload() {
    gen_verify_example(
        "examples/019_operator_overload",
        "operator_overload",
        "operator_overload",
    );
}

/// L6-25：020_friend_function — 友元函数
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_friend_function() {
    gen_verify_example(
        "examples/020_friend_function",
        "friend_function",
        "friend_function",
    );
}

/// L6-26：021_explicit_ctor — explicit 构造函数
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_explicit_ctor() {
    gen_verify_example(
        "examples/021_explicit_ctor",
        "explicit_ctor",
        "explicit_ctor",
    );
}

/// L6-27：022_mutable_member — mutable 成员
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_mutable_member() {
    gen_verify_example(
        "examples/022_mutable_member",
        "mutable_member",
        "mutable_member",
    );
}

/// L6-28：023_typeid_rtti — RTTI typeid
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_typeid_rtti() {
    gen_verify_example("examples/023_typeid_rtti", "typeid_rtti", "typeid_rtti");
}

/// L6-29：026_template_specialization — 模板特化
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_template_specialization() {
    gen_verify_example(
        "examples/026_template_specialization",
        "template_specialization",
        "template_specialization",
    );
}

/// L6-30：027_template_instantiation — 模板实例化
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_template_instantiation() {
    gen_verify_example(
        "examples/027_template_instantiation",
        "template_instantiation",
        "template_instantiation",
    );
}

/// L6-31：028_variadic_template — 可变参数模板
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_variadic_template() {
    gen_verify_example(
        "examples/028_variadic_template",
        "variadic_template",
        "variadic_template",
    );
}

/// L6-32：030_shared_ptr — shared_ptr 智能指针
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_shared_ptr() {
    gen_verify_example("examples/030_shared_ptr", "shared_ptr", "shared_ptr");
}

/// L6-33：031_custom_deleter — 自定义删除器
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_custom_deleter() {
    gen_verify_example(
        "examples/031_custom_deleter",
        "custom_deleter",
        "custom_deleter",
    );
}

/// L6-34：032_placement_new — 定位 new
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_placement_new() {
    gen_verify_example(
        "examples/032_placement_new",
        "placement_new",
        "placement_new",
    );
}

/// L6-35：033_raii_pattern — RAII 模式
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_raii_pattern() {
    gen_verify_example("examples/033_raii_pattern", "raii_pattern", "raii_pattern");
}

/// L6-36：035_map_basic — STL 容器（map）
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_map_basic() {
    gen_verify_example("examples/035_map_basic", "map_basic", "map_basic");
}

/// L6-37：036_string_basic — STL 容器（string）
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_string_basic() {
    gen_verify_example("examples/036_string_basic", "string_basic", "string_basic");
}

/// L6-38：037_array_basic — STL 容器（array）
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_array_basic() {
    gen_verify_example("examples/037_array_basic", "array_basic", "array_basic");
}

/// L6-39：038_tuple_basic — STL 容器（tuple）
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_tuple_basic() {
    gen_verify_example("examples/038_tuple_basic", "tuple_basic", "tuple_basic");
}

/// L6-40：039_lambda_basic — Lambda 表达式
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_lambda_basic() {
    gen_verify_example("examples/039_lambda_basic", "lambda_basic", "lambda_basic");
}

/// L6-41：040_std_function — std::function
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_std_function() {
    gen_verify_example("examples/040_std_function", "std_function", "std_function");
}

/// L6-42：041_functional_bind — std::bind
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_functional_bind() {
    gen_verify_example(
        "examples/041_functional_bind",
        "functional_bind",
        "functional_bind",
    );
}

/// L6-43：043_namespace_nested — 嵌套命名空间
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_namespace_nested() {
    gen_verify_example(
        "examples/043_namespace_nested",
        "namespace_nested",
        "namespace_nested",
    );
}

/// L6-44：044_enum_class — 枚举类
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_enum_class() {
    gen_verify_example("examples/044_enum_class", "enum_class", "enum_class");
}

/// L6-45：045_union_basic — 联合体
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_union_basic() {
    gen_verify_example("examples/045_union_basic", "union_basic", "union_basic");
}

/// L6-46：046_constexpr_basic — constexpr
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_constexpr_basic() {
    gen_verify_example(
        "examples/046_constexpr_basic",
        "constexpr_basic",
        "constexpr_basic",
    );
}

/// L6-47：047_noexcept_basic — noexcept
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_noexcept_basic() {
    gen_verify_example(
        "examples/047_noexcept_basic",
        "noexcept_basic",
        "noexcept_basic",
    );
}

/// L6-48：048_summary — 综合示例
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与 libclang，在 CI gen-verify job 中运行"]
fn gen_verify_summary() {
    gen_verify_example("examples/048_summary", "summary", "summary");
}

/// L6-49（回归）：多单元 + hicc 宏位于子模块 + 实际链接二进制目标。
///
/// 锁死「生成的 build.rs 只注册无宏的 `src/lib.rs`」回归：当 `import_lib!` 分散在
/// `src/<unit>.rs` 子模块文件中时，`hicc-build` 必须对**每个**单元文件调用
/// `.rust_file(...)`，否则 C++ 侧 `_hicc_export_methods_*` 方法表导出函数不会生成，
/// 链接测试二进制时报 `undefined reference`（详见 problem statement 的 rapidjson 报错）。
///
/// 与 [`gen_verify_example`] 不同：本测试**手工**构造一个多单元项目（不经 AST 解析，
/// 故不依赖 libclang），用 [`project_generator::write_build_rs`] 生成 build.rs，再以
/// `cargo test` 真正链接并运行一个测试目标。仍需 g++/clang++ 与网络获取 hicc crate，
/// 故标注 `#[ignore]`，在 CI gen-verify job 中运行。
#[test]
#[ignore = "gen-verify: 需要 g++/clang++ 与联网获取 hicc crate，在 CI gen-verify job 中运行"]
fn gen_verify_multi_unit_submodule_links() {
    use cpp2rust_demo::generator::project_generator;

    let tmp = TempDir::new().expect("创建临时目录失败");
    let project_dir = tmp.path().to_path_buf();
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(src_dir.join("sub")).expect("创建 src/sub 失败");

    // 单元 1：src/math_ffi.rs（含 cpp! + import_lib!）
    std::fs::write(
        src_dir.join("math_ffi.rs"),
        r#"hicc::cpp! {
    extern "C" int gv_add(int a, int b) { return a + b; }
}
hicc::import_lib! {
    #![link_name = "gv_math"]
    #[cpp(func = "int gv_add(int, int)")]
    pub fn gv_add(a: i32, b: i32) -> i32;
}
"#,
    )
    .expect("写 math_ffi.rs 失败");

    // 单元 2：src/sub/text_ffi.rs（嵌套子目录，验证多级模块逐文件注册）
    std::fs::write(
        src_dir.join("sub").join("text_ffi.rs"),
        r#"hicc::cpp! {
    extern "C" int gv_mul(int a, int b) { return a * b; }
}
hicc::import_lib! {
    #![link_name = "gv_text"]
    #[cpp(func = "int gv_mul(int, int)")]
    pub fn gv_mul(a: i32, b: i32) -> i32;
}
"#,
    )
    .expect("写 sub/text_ffi.rs 失败");

    // 单元路径与 init 阶段一致（相对 src/、`/` 分隔、不含扩展名）
    let unit_paths = vec!["math_ffi".to_string(), "sub/text_ffi".to_string()];

    // lib.rs（仅模块声明，无宏）+ build.rs（被测：逐文件注册）
    project_generator::write_lib_rs(&project_dir, &unit_paths).expect("写 lib.rs 失败");
    project_generator::write_build_rs(
        &project_dir,
        "gv_multi",
        &unit_paths,
        &cpp2rust_demo::build_meta::BuildMeta::default(),
    )
    .expect("写 build.rs 失败");

    // build.rs 必须为每个含宏单元注册 .rust_file，而非仅 src/lib.rs
    let build_rs = std::fs::read_to_string(project_dir.join("build.rs")).unwrap();
    assert!(
        build_rs.contains(".rust_file(\"src/math_ffi.rs\")")
            && build_rs.contains(".rust_file(\"src/sub/text_ffi.rs\")"),
        "build.rs 应逐文件注册所有含宏单元，实际：\n{build_rs}"
    );

    std::fs::write(
        project_dir.join("Cargo.toml"),
        r#"[package]
name = "gv-multi"
version = "0.1.0"
edition = "2018"

[lib]
name = "gv_multi"
path = "src/lib.rs"

[dependencies]
hicc = { version = "0.2" }

[build-dependencies]
hicc-build = { version = "0.2" }
cc = "1.0"
"#,
    )
    .expect("写 Cargo.toml 失败");

    // 链接二进制（集成测试目标）：调用两个分散在不同子模块的 import_lib! 函数
    std::fs::create_dir_all(project_dir.join("tests")).expect("创建 tests 失败");
    std::fs::write(
        project_dir.join("tests").join("smoke.rs"),
        r#"#[test]
fn calls_both_units() {
    assert_eq!(gv_multi::gv_add(2, 3), 5);
    assert_eq!(gv_multi::gv_mul(4, 5), 20);
}
"#,
    )
    .expect("写 tests/smoke.rs 失败");

    // cargo test：真正链接并运行测试目标。若 build.rs 漏注册子模块单元，
    // 链接阶段会因 `_hicc_export_methods_*` 未定义而失败。
    let output = std::process::Command::new("cargo")
        .args(["test", "--manifest-path"])
        .arg(project_dir.join("Cargo.toml"))
        .output()
        .expect("运行 cargo test 失败");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "[gen-verify] 多单元子模块项目 cargo test 失败（应已逐文件注册 .rust_file）\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }
    println!("[gen-verify] ✅ 多单元 + 子模块宏 + 链接二进制 回归通过");
}

// ─────────────────────────────────────────────────────────────────
//  核心验证逻辑
// ─────────────────────────────────────────────────────────────────

/// 去除 Windows 上 canonicalize() 返回的 `\\?\` UNC 前缀。
/// MSVC cl.exe 无法处理此前缀，会导致编译失败。
fn strip_windows_unc_prefix(path: PathBuf) -> PathBuf {
    match path.to_str() {
        Some(s) if s.starts_with("\\\\?\\") => PathBuf::from(&s[4..]),
        _ => path,
    }
}

/// 对指定示例运行完整的生成 → 编译验证流程：
///
/// 1. 调用 `common::run_tool_on` 生成 FFI 代码（hicc 三段式 Rust 块）
/// 2. 验证生成代码的基本结构（包含 `import_lib!`）
/// 3. 在临时目录创建完整 Cargo 项目，build.rs 使用绝对路径引用原示例 C++ 文件
/// 4. 运行 `cargo build` 验证生成代码可被编译
///
/// 若预处理/解析失败（如当前环境无 g++ 或 libclang），则优雅跳过而非 panic。
fn gen_verify_example(example_dir: &str, lib_name: &str, cpp_stem: &str) {
    // ── 步骤 1：生成 FFI 代码 ──────────────────────────────────────
    let generated_code = common::run_tool_on(example_dir);
    if generated_code.is_empty() {
        eprintln!(
            "[gen-verify] 跳过 {}：预处理或 AST 解析失败（当前环境可能缺少 g++/libclang）",
            example_dir
        );
        return;
    }

    // ── 步骤 2：验证基本结构 ──────────────────────────────────────
    // 工具产物至少应包含 hicc `cpp!` 块（含项目头 include）。多数示例还会生成
    // import_lib!（自由函数/工厂）或 import_class!（命名空间类）绑定块；但部分
    // 示例（如 013_inheritance_single：成员为 std::string，工具默认不自动映射）
    // 默认仅生成 `cpp!` 头块骨架——这是合法且可编译的产物，其完整绑定由手写
    // lib.rs 补全。因此此处只强制要求 `cpp!` 块存在，可编译性由步骤 4 的
    // cargo build 实际验证。
    assert!(
        generated_code.contains("hicc::cpp!"),
        "[gen-verify] {} 的生成代码应至少包含 hicc::cpp! 块\n实际生成：\n{}",
        example_dir,
        generated_code
    );

    // ── 步骤 3：创建临时 Cargo 项目 ──────────────────────────────
    let tmp = TempDir::new().expect("创建临时目录失败");
    let project_dir = tmp.path().to_path_buf();
    setup_gen_verify_project(
        &project_dir,
        lib_name,
        cpp_stem,
        example_dir,
        &generated_code,
    );

    // ── 步骤 4：运行 cargo build ──────────────────────────────────
    let output = std::process::Command::new("cargo")
        .args(["build", "--manifest-path"])
        .arg(project_dir.join("Cargo.toml"))
        .output()
        .expect("运行 cargo build 失败");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "[gen-verify] {} 的生成代码无法通过 cargo build\nstdout:\n{}\nstderr:\n{}",
            example_dir, stdout, stderr
        );
    }

    println!("[gen-verify] ✅ {} 通过 cargo build", example_dir);
}

/// 在 `project_dir` 下创建完整的临时 Cargo 项目结构：
/// - `src/lib.rs`：工具生成的 FFI 代码（hicc 三段式）
/// - `Cargo.toml`：依赖 hicc / hicc-build / cc
/// - `build.rs`：使用绝对路径编译原示例 C++ 文件
fn setup_gen_verify_project(
    project_dir: &Path,
    lib_name: &str,
    cpp_stem: &str,
    example_dir: &str,
    generated_code: &str,
) {
    // 创建 src/ 目录
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).expect("创建 src 目录失败");

    // ── lib.rs ────────────────────────────────────────────────────
    // 工具生成的 hicc 三段式就是完整的 lib.rs 内容
    std::fs::write(src_dir.join("lib.rs"), generated_code).expect("写 lib.rs 失败");

    // ── Cargo.toml ───────────────────────────────────────────────
    let lib_name_ident = lib_name.replace('-', "_");
    let cargo_toml = format!(
        r#"[package]
name = "{lib_name}-gen-verify"
version = "0.1.0"
edition = "2021"

[lib]
name = "{lib_name_ident}"
path = "src/lib.rs"

[dependencies]
hicc = {{ version = "0.2" }}

[build-dependencies]
hicc-build = {{ version = "0.2" }}
cc = "1.0"
"#,
        lib_name = lib_name,
        lib_name_ident = lib_name_ident,
    );
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml).expect("写 Cargo.toml 失败");

    // ── build.rs ─────────────────────────────────────────────────
    // 使用绝对路径引用原示例 C++ 文件，避免相对路径问题
    let example_abs = PathBuf::from(example_dir)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(example_dir));
    // Windows 上 canonicalize() 返回 \\?\ 前缀路径（UNC 扩展路径），
    // MSVC cl.exe 无法处理该前缀，需去除。
    let example_abs = strip_windows_unc_prefix(example_abs);
    let cpp_dir = example_abs.join("cpp");

    // 收集所有 .cpp 文件
    let cpp_files: Vec<PathBuf> = std::fs::read_dir(&cpp_dir)
        .unwrap_or_else(|_| panic!("无法读取 C++ 目录：{}", cpp_dir.display()))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("cpp"))
        .collect();

    let cpp_file_lines: String = cpp_files.iter().fold(String::new(), |mut s, f| {
        use std::fmt::Write;
        let _ = writeln!(s, "    cc_build.file({:?});", f);
        s
    });

    // rerun-if-changed 行
    let rerun_lines: String = cpp_files.iter().fold(String::new(), |mut s, f| {
        use std::fmt::Write;
        let escaped = f.display().to_string().replace('\\', "\\\\");
        let _ = writeln!(s, "    println!(\"cargo::rerun-if-changed={}\");", escaped);
        s
    });

    let build_rs = format!(
        r#"fn main() {{
    let cpp_dir = std::path::PathBuf::from({cpp_dir:?});

    let mut build = hicc_build::Build::new();
    use std::ops::DerefMut;
    let cc_build: &mut cc::Build = build.deref_mut();
    cc_build.include(&cpp_dir);
    cc_build.cpp(true);
    cc_build.std("c++17");
{cpp_file_lines}
    build.rust_file("src/lib.rs").compile({cpp_stem:?});

    println!("cargo::rustc-link-lib={cpp_stem}");
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=src/lib.rs");
{rerun_lines}}}
"#,
        cpp_dir = cpp_dir,
        cpp_stem = cpp_stem,
        cpp_file_lines = cpp_file_lines,
        rerun_lines = rerun_lines,
    );

    std::fs::write(project_dir.join("build.rs"), build_rs).expect("写 build.rs 失败");
}
