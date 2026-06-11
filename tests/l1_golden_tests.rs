mod common;

macro_rules! golden_test {
    ($name:ident, $example:literal) => {
        #[test]
        #[cfg_attr(
            not(feature = "full-test"),
            ignore = "requires libclang; run with --features full-test --test-threads=1"
        )]
        fn $name() {
            let example_dir = concat!("examples/", $example);
            let generated = common::run_tool_on(example_dir);
            let golden_raw = common::read_golden(example_dir, "rust_hicc/src/main.rs");
            let golden = common::extract_hicc_blocks(&golden_raw);
            assert_eq!(
                common::normalize(&generated),
                common::normalize(&golden),
                "FFI scaffold mismatch for {}",
                $example
            );
        }
    };
}

// Windows libclang 与 Linux libclang 对某些构造（inline 函数、typedef、
// union class、函数顺序等）有不同的 AST 处理，导致生成结果与 Linux golden
// 文件不一致。对这些已知差异测试，仅在非 Windows 平台运行。
macro_rules! golden_test_unix_only {
    ($name:ident, $example:literal) => {
        #[test]
        #[cfg(not(windows))]
        #[cfg_attr(
            not(feature = "full-test"),
            ignore = "requires libclang; run with --features full-test --test-threads=1"
        )]
        fn $name() {
            let example_dir = concat!("examples/", $example);
            let generated = common::run_tool_on(example_dir);
            let golden_raw = common::read_golden(example_dir, "rust_hicc/src/main.rs");
            let golden = common::extract_hicc_blocks(&golden_raw);
            assert_eq!(
                common::normalize(&generated),
                common::normalize(&golden),
                "FFI scaffold mismatch for {}",
                $example
            );
        }
    };
}

// Phase E 升级示例（lib.rs + main.rs 结构）：从 lib.rs 读取黄金内容。
// lib.rs 中函数使用 `pub unsafe fn` 以支持集成测试访问，而工具生成器暂输出
// `unsafe fn`（无 pub），因此比较前通过 strip_pub_visibility 规范化可见性修饰符。
macro_rules! golden_test_lib {
    ($name:ident, $example:literal) => {
        #[test]
        #[cfg_attr(
            not(feature = "full-test"),
            ignore = "requires libclang; run with --features full-test --test-threads=1"
        )]
        fn $name() {
            let example_dir = concat!("examples/", $example);
            let generated = common::run_tool_on(example_dir);
            let golden_raw = common::read_golden(example_dir, "rust_hicc/src/lib.rs");
            // 从 lib.rs 提取 hicc 块后，规范化 pub 可见性再比较
            let golden = common::extract_hicc_blocks(&golden_raw);
            assert_eq!(
                common::normalize(&generated),
                common::normalize(&common::strip_pub_visibility(&golden)),
                "FFI scaffold mismatch for {}",
                $example
            );
        }
    };
}

golden_test!(test_001_hello_world, "001_hello_world");
golden_test!(test_002_function_overload, "002_function_overload");
golden_test!(test_003_default_args, "003_default_args");
golden_test_unix_only!(test_004_inline_functions, "004_inline_functions");
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
golden_test_lib!(test_015_virtual_basic, "015_virtual_basic");
golden_test_lib!(test_016_virtual_pure, "016_virtual_pure");
golden_test_lib!(test_017_virtual_override, "017_virtual_override");
golden_test_lib!(test_018_virtual_diamond, "018_virtual_diamond");
golden_test!(test_019_operator_overload, "019_operator_overload");
golden_test!(test_020_friend_function, "020_friend_function");
golden_test!(test_021_explicit_ctor, "021_explicit_ctor");
golden_test!(test_022_mutable_member, "022_mutable_member");
golden_test_lib!(test_023_typeid_rtti, "023_typeid_rtti");
golden_test_lib!(test_024_template_function, "024_template_function");
golden_test_lib!(test_025_template_class, "025_template_class");
golden_test_lib!(test_026_template_specialization, "026_template_specialization");
golden_test_lib!(test_027_template_instantiation, "027_template_instantiation");
golden_test!(test_028_variadic_template, "028_variadic_template");
golden_test!(test_029_unique_ptr, "029_unique_ptr");
golden_test!(test_030_shared_ptr, "030_shared_ptr");
golden_test_unix_only!(test_031_custom_deleter, "031_custom_deleter");
golden_test!(test_032_placement_new, "032_placement_new");
golden_test!(test_033_raii_pattern, "033_raii_pattern");
golden_test_lib!(test_034_vector_basic, "034_vector_basic");
golden_test_lib!(test_035_map_basic, "035_map_basic");
golden_test_lib!(test_036_string_basic, "036_string_basic");
golden_test_lib!(test_037_array_basic, "037_array_basic");
golden_test_lib!(test_038_tuple_basic, "038_tuple_basic");
golden_test_unix_only!(test_039_lambda_basic, "039_lambda_basic");
golden_test!(test_040_std_function, "040_std_function");
golden_test!(test_041_functional_bind, "041_functional_bind");
golden_test!(test_042_exception_basic, "042_exception_basic");
golden_test_unix_only!(test_043_namespace_nested, "043_namespace_nested");
golden_test!(test_044_enum_class, "044_enum_class");
golden_test_unix_only!(test_045_union_basic, "045_union_basic");
golden_test!(test_046_constexpr_basic, "046_constexpr_basic");
golden_test_unix_only!(test_047_noexcept_basic, "047_noexcept_basic");
golden_test!(test_048_summary, "048_summary");

// ── 降级标记断言：直接验证 cpp2rust-todo[TAG] 是否被正确生成 ──────────────────
//
// 以下测试不依赖 normalize 的注释剥除逻辑，直接检查原始生成代码是否包含
// 对应的降级标记，以防生成器逻辑回归导致标记被静默丢失。

macro_rules! todo_tag_test {
    ($name:ident, $example:literal, $tag:literal) => {
        #[test]
        #[cfg_attr(
            not(feature = "full-test"),
            ignore = "requires libclang; run with --features full-test --test-threads=1"
        )]
        fn $name() {
            let example_dir = concat!("examples/", $example);
            let generated = common::run_tool_on(example_dir);
            common::assert_contains_todo_tag(&generated, $tag, $example);
        }
    };
}

todo_tag_test!(test_031_todo_fp, "031_custom_deleter", "FP");
todo_tag_test!(test_039_todo_fp, "039_lambda_basic", "FP");
todo_tag_test!(test_040_todo_fp, "040_std_function", "FP");
#[cfg(not(windows))]
todo_tag_test!(test_047_todo_fp, "047_noexcept_basic", "FP");
