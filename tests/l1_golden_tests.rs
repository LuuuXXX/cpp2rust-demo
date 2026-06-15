mod common;

// Phase E 升级示例（lib.rs + main.rs 结构）：从 lib.rs 读取黄金内容。
// 工具生成器输出 `pub fn`（含 pub），与 lib.rs 黄金文件中的可见性修饰符一致。
macro_rules! golden_test_lib {
    ($name:ident, $example:literal) => {
        golden_test_lib!($name, $example, "rust_hicc/src/lib.rs");
    };
    // 当 lib.rs 含有超出工具自动生成范围的手动修改时，可通过 $golden_file 指向
    // 单独维护的支架黄金文件（lib_scaffold.rs），以避免手动修改干扰自动生成验证。
    ($name:ident, $example:literal, $golden_file:literal) => {
        #[test]
        #[cfg_attr(
            not(feature = "full-test"),
            ignore = "requires libclang; run with --features full-test --test-threads=1"
        )]
        fn $name() {
            let example_dir = concat!("examples/", $example);
            let generated = common::run_tool_on(example_dir);
            let golden_raw = common::read_golden(example_dir, $golden_file);
            // 从黄金文件提取 hicc 块后，规范化 pub 可见性再比较
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

// Unix-only 版本的 golden_test_lib：Windows libclang 对某些构造（inline 函数、
// typedef、union class 等）有不同的 AST 处理，仅在非 Windows 平台运行。
macro_rules! golden_test_lib_unix_only {
    ($name:ident, $example:literal) => {
        golden_test_lib_unix_only!($name, $example, "rust_hicc/src/lib.rs");
    };
    ($name:ident, $example:literal, $golden_file:literal) => {
        #[test]
        #[cfg(not(windows))]
        #[cfg_attr(
            not(feature = "full-test"),
            ignore = "requires libclang; run with --features full-test --test-threads=1"
        )]
        fn $name() {
            let example_dir = concat!("examples/", $example);
            let generated = common::run_tool_on(example_dir);
            let golden_raw = common::read_golden(example_dir, $golden_file);
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

// v7：当工具默认输出含生成器自带 `pub` 修饰的骨架（如模板函数 `pub unsafe fn do_swap`、
// 模板构造工厂、代理工厂、dynamic_cast 等）时，使用独立维护的支架黄金文件
// （`lib_scaffold.rs`）做精确比对，用于校验工具默认产物。
macro_rules! golden_test_scaffold {
    ($name:ident, $example:literal) => {
        golden_test_scaffold!($name, $example, "rust_hicc/src/lib_scaffold.rs");
    };
    ($name:ident, $example:literal, $golden_file:literal) => {
        #[test]
        #[cfg_attr(
            not(feature = "full-test"),
            ignore = "requires libclang; run with --features full-test --test-threads=1"
        )]
        fn $name() {
            let example_dir = concat!("examples/", $example);
            let generated = common::run_tool_on(example_dir);
            let golden_raw = common::read_golden(example_dir, $golden_file);
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

golden_test_lib!(test_001_hello_world, "001_hello_world");
golden_test_lib!(test_002_function_overload, "002_function_overload");
golden_test_lib!(test_003_default_args, "003_default_args");
golden_test_lib_unix_only!(test_004_inline_functions, "004_inline_functions");
golden_test_lib!(test_005_variadic_functions, "005_variadic_functions");
golden_test_scaffold!(test_006_class_basic, "006_class_basic");
golden_test_lib!(test_007_class_constructor, "007_class_constructor");
golden_test_lib!(test_008_class_copy, "008_class_copy");
golden_test_lib!(test_009_class_move, "009_class_move");
golden_test_lib!(test_010_class_static, "010_class_static");
golden_test_lib!(test_011_class_const, "011_class_const");
golden_test_lib!(test_012_class_volatile, "012_class_volatile");
golden_test_lib!(test_013_inheritance_single, "013_inheritance_single");
golden_test_lib!(test_014_inheritance_multiple, "014_inheritance_multiple");
golden_test_lib!(test_015_virtual_basic, "015_virtual_basic");
golden_test_lib!(test_016_virtual_pure, "016_virtual_pure");
golden_test_lib!(test_017_virtual_override, "017_virtual_override");
// lib.rs 含有针对 hicc member_addr 截断 this 偏移量问题的手动包装函数修复，
// 使用独立的 lib_scaffold.rs 作为工具自动生成部分的黄金比对文件。
golden_test_lib!(
    test_018_virtual_diamond,
    "018_virtual_diamond",
    "rust_hicc/src/lib_scaffold.rs"
);
golden_test_lib!(test_019_operator_overload, "019_operator_overload");
golden_test_lib!(test_020_friend_function, "020_friend_function");
golden_test_lib!(test_021_explicit_ctor, "021_explicit_ctor");
golden_test_lib!(test_022_mutable_member, "022_mutable_member");
golden_test_lib!(test_023_typeid_rtti, "023_typeid_rtti");
golden_test_scaffold!(test_024_template_function, "024_template_function");
golden_test_lib!(test_025_template_class, "025_template_class");
golden_test_lib!(
    test_026_template_specialization,
    "026_template_specialization"
);
golden_test_lib!(
    test_027_template_instantiation,
    "027_template_instantiation"
);
golden_test_lib!(test_028_variadic_template, "028_variadic_template");
golden_test_lib!(test_029_unique_ptr, "029_unique_ptr");
golden_test_lib!(test_030_shared_ptr, "030_shared_ptr");
golden_test_lib_unix_only!(test_031_custom_deleter, "031_custom_deleter");
golden_test_lib!(test_032_placement_new, "032_placement_new");
golden_test_lib!(test_033_raii_pattern, "033_raii_pattern");
golden_test_lib!(test_034_vector_basic, "034_vector_basic");
golden_test_lib!(test_035_map_basic, "035_map_basic");
golden_test_lib!(test_036_string_basic, "036_string_basic");
golden_test_lib!(test_037_array_basic, "037_array_basic");
golden_test_lib!(test_038_tuple_basic, "038_tuple_basic");
golden_test_lib_unix_only!(test_039_lambda_basic, "039_lambda_basic");
golden_test_lib!(test_040_std_function, "040_std_function");
golden_test_lib!(test_041_functional_bind, "041_functional_bind");
golden_test_lib!(test_042_exception_basic, "042_exception_basic");
golden_test_lib_unix_only!(test_043_namespace_nested, "043_namespace_nested");
golden_test_lib!(test_044_enum_class, "044_enum_class");
golden_test_lib_unix_only!(test_045_union_basic, "045_union_basic");
golden_test_lib!(test_046_constexpr_basic, "046_constexpr_basic");
golden_test_lib_unix_only!(test_047_noexcept_basic, "047_noexcept_basic");
golden_test_lib!(test_048_summary, "048_summary");

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
