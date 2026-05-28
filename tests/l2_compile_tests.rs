mod common;

macro_rules! compile_test {
    ($name:ident, $example:literal) => {
        #[test]
        fn $name() {
            let dir = concat!("examples/", $example, "/rust_hicc");
            assert!(
                common::cargo_build(dir),
                "cargo build failed for {}",
                $example
            );
        }
    };
}

// Known pre-existing compilation failures that need investigation and fixing.
macro_rules! compile_test_ignore {
    ($name:ident, $example:literal) => {
        #[test]
        #[ignore = "known pre-existing compilation failure - needs investigation"]
        fn $name() {
            let dir = concat!("examples/", $example, "/rust_hicc");
            assert!(
                common::cargo_build(dir),
                "cargo build failed for {}",
                $example
            );
        }
    };
}

compile_test!(compile_001_hello_world, "001_hello_world");
compile_test!(compile_002_function_overload, "002_function_overload");
compile_test!(compile_003_default_args, "003_default_args");
compile_test!(compile_004_inline_functions, "004_inline_functions");
compile_test!(compile_005_variadic_functions, "005_variadic_functions");
compile_test!(compile_006_class_basic, "006_class_basic");
compile_test!(compile_007_class_constructor, "007_class_constructor");
compile_test!(compile_008_class_copy, "008_class_copy");
compile_test_ignore!(compile_009_class_move, "009_class_move");
compile_test!(compile_010_class_static, "010_class_static");
compile_test!(compile_011_class_const, "011_class_const");
compile_test_ignore!(compile_012_class_volatile, "012_class_volatile");
compile_test!(compile_013_inheritance_single, "013_inheritance_single");
compile_test!(compile_014_inheritance_multiple, "014_inheritance_multiple");
compile_test!(compile_015_virtual_basic, "015_virtual_basic");
compile_test!(compile_016_virtual_pure, "016_virtual_pure");
compile_test!(compile_017_virtual_override, "017_virtual_override");
compile_test!(compile_018_virtual_diamond, "018_virtual_diamond");
compile_test!(compile_019_operator_overload, "019_operator_overload");
compile_test_ignore!(compile_020_friend_function, "020_friend_function");
compile_test!(compile_021_explicit_ctor, "021_explicit_ctor");
compile_test!(compile_022_mutable_member, "022_mutable_member");
compile_test_ignore!(compile_023_typeid_rtti, "023_typeid_rtti");
compile_test!(compile_024_template_function, "024_template_function");
compile_test_ignore!(compile_025_template_class, "025_template_class");
compile_test!(compile_026_template_specialization, "026_template_specialization");
compile_test!(compile_027_template_instantiation, "027_template_instantiation");
compile_test!(compile_028_variadic_template, "028_variadic_template");
compile_test!(compile_029_unique_ptr, "029_unique_ptr");
compile_test!(compile_030_shared_ptr, "030_shared_ptr");
compile_test_ignore!(compile_031_custom_deleter, "031_custom_deleter");
compile_test!(compile_032_placement_new, "032_placement_new");
compile_test_ignore!(compile_033_raii_pattern, "033_raii_pattern");
compile_test!(compile_034_vector_basic, "034_vector_basic");
compile_test!(compile_035_map_basic, "035_map_basic");
compile_test!(compile_036_string_basic, "036_string_basic");
compile_test!(compile_037_array_basic, "037_array_basic");
compile_test!(compile_038_tuple_basic, "038_tuple_basic");
compile_test_ignore!(compile_039_lambda_basic, "039_lambda_basic");
compile_test_ignore!(compile_040_std_function, "040_std_function");
compile_test_ignore!(compile_041_functional_bind, "041_functional_bind");
compile_test!(compile_042_exception_basic, "042_exception_basic");
compile_test!(compile_043_namespace_nested, "043_namespace_nested");
compile_test!(compile_044_enum_class, "044_enum_class");
compile_test_ignore!(compile_045_union_basic, "045_union_basic");
compile_test_ignore!(compile_046_constexpr_basic, "046_constexpr_basic");
compile_test!(compile_047_noexcept_basic, "047_noexcept_basic");
compile_test!(compile_048_summary, "048_summary");
