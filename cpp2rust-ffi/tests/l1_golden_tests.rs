//! L1 黄金文件测试
//! 工具生成的 main.rs 与仓库中 rust_hicc/src/main.rs 进行对比
//! 初始状态：工具未实现时，所有测试均为 FAIL（符合预期）

mod common;

use common::{normalize, read_golden};
use std::path::PathBuf;
use std::process::Command;

/// 运行工具对指定示例生成代码，返回生成的内容
fn run_tool_on(example: &str) -> String {
    let example_dir = common::example_dir(example);
    let cpp_dir = example_dir.join("cpp");

    // 收集所有 .cpp 文件
    let cpp_files: Vec<PathBuf> = std::fs::read_dir(&cpp_dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.extension().map_or(false, |e| {
                        matches!(e.to_str().unwrap_or(""), "cpp" | "cc" | "cxx")
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    if cpp_files.is_empty() {
        return String::new();
    }

    // 调用 cpp2rust-ffi generate 命令（使用第一个 .cpp 文件）
    let tool_bin = PathBuf::from(env!("CARGO_BIN_EXE_cpp2rust-ffi"));

    let output = Command::new(&tool_bin)
        .args(["generate", "--input"])
        .arg(&cpp_files[0])
        .output()
        .expect("Failed to run cpp2rust-ffi");

    String::from_utf8_lossy(&output.stdout).to_string()
}

macro_rules! golden_test {
    ($name:ident, $example:literal) => {
        #[test]
        #[ignore = "L1 tests require tool implementation (initially FAIL)"]
        fn $name() {
            let generated = run_tool_on($example);
            let golden = read_golden($example);
            assert_eq!(
                normalize(&generated),
                normalize(&golden),
                "Golden file mismatch for {}",
                $example
            );
        }
    };
}

golden_test!(test_001_hello_world, "001_hello_world");
golden_test!(test_002_function_overload, "002_function_overload");
golden_test!(test_003_default_args, "003_default_args");
golden_test!(test_004_inline_functions, "004_inline_functions");
golden_test!(test_005_variadic_functions, "005_variadic_functions");
golden_test!(test_006_class_basic, "006_class_basic");
golden_test!(test_007_class_constructor, "007_class_constructor");
golden_test!(test_008_class_copy, "008_class_copy");
golden_test!(test_009_class_move, "009_class_move");
golden_test!(test_010_class_static, "010_class_static");
golden_test!(test_011_class_const, "011_class_const");
golden_test!(test_012_class_volatile, "012_class_volatile");
golden_test!(test_013_inheritance_single, "013_inheritance_single");
golden_test!(test_014_inheritance_multiple, "014_inheritance_multiple");
golden_test!(test_015_virtual_basic, "015_virtual_basic");
golden_test!(test_016_virtual_pure, "016_virtual_pure");
golden_test!(test_017_virtual_override, "017_virtual_override");
golden_test!(test_018_virtual_diamond, "018_virtual_diamond");
golden_test!(test_019_operator_overload, "019_operator_overload");
golden_test!(test_020_friend_function, "020_friend_function");
golden_test!(test_021_explicit_ctor, "021_explicit_ctor");
golden_test!(test_022_mutable_member, "022_mutable_member");
golden_test!(test_023_typeid_rtti, "023_typeid_rtti");
golden_test!(test_024_template_function, "024_template_function");
golden_test!(test_025_template_class, "025_template_class");
golden_test!(test_026_template_specialization, "026_template_specialization");
golden_test!(test_027_template_instantiation, "027_template_instantiation");
golden_test!(test_028_variadic_template, "028_variadic_template");
golden_test!(test_029_unique_ptr, "029_unique_ptr");
golden_test!(test_030_shared_ptr, "030_shared_ptr");
golden_test!(test_031_custom_deleter, "031_custom_deleter");
golden_test!(test_032_placement_new, "032_placement_new");
golden_test!(test_033_raii_pattern, "033_raii_pattern");
golden_test!(test_034_vector_basic, "034_vector_basic");
golden_test!(test_035_map_basic, "035_map_basic");
golden_test!(test_036_string_basic, "036_string_basic");
golden_test!(test_037_array_basic, "037_array_basic");
golden_test!(test_038_tuple_basic, "038_tuple_basic");
golden_test!(test_039_lambda_basic, "039_lambda_basic");
golden_test!(test_040_std_function, "040_std_function");
golden_test!(test_041_functional_bind, "041_functional_bind");
golden_test!(test_042_exception_basic, "042_exception_basic");
golden_test!(test_043_namespace_nested, "043_namespace_nested");
golden_test!(test_044_enum_class, "044_enum_class");
golden_test!(test_045_union_basic, "045_union_basic");
golden_test!(test_046_constexpr_basic, "046_constexpr_basic");
golden_test!(test_047_noexcept_basic, "047_noexcept_basic");
golden_test!(test_048_summary, "048_summary");
