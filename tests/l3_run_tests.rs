mod common;

macro_rules! run_test {
    ($name:ident, $example:literal) => {
        #[test]
        #[ignore = "Requires full runtime environment"]
        fn $name() {
            let dir = concat!("examples/", $example, "/rust_hicc");
            let readme = concat!("examples/", $example, "/README.md");
            let output = common::cargo_run(dir);
            let expected = common::parse_readme_run_result(readme);
            let actual = output.trim().to_string();
            let expected_trimmed = expected.trim().to_string();
            assert!(
                common::compare_run_output(&actual, &expected_trimmed),
                "cargo run output mismatch for {}\n\n=== actual ===\n{}\n\n=== expected ===\n{}",
                $example,
                actual,
                expected_trimmed
            );
        }
    };
}

run_test!(run_001_hello_world, "001_hello_world");
run_test!(run_002_function_overload, "002_function_overload");
run_test!(run_003_default_args, "003_default_args");
run_test!(run_004_inline_functions, "004_inline_functions");
run_test!(run_005_variadic_functions, "005_variadic_functions");
run_test!(run_006_class_basic, "006_class_basic");
run_test!(run_007_class_constructor, "007_class_constructor");
run_test!(run_008_class_copy, "008_class_copy");
run_test!(run_009_class_move, "009_class_move");
run_test!(run_010_class_static, "010_class_static");
run_test!(run_011_class_const, "011_class_const");
run_test!(run_012_class_volatile, "012_class_volatile");
run_test!(run_013_inheritance_single, "013_inheritance_single");
run_test!(run_014_inheritance_multiple, "014_inheritance_multiple");
run_test!(run_015_virtual_basic, "015_virtual_basic");
run_test!(run_016_virtual_pure, "016_virtual_pure");
run_test!(run_017_virtual_override, "017_virtual_override");
run_test!(run_018_virtual_diamond, "018_virtual_diamond");
run_test!(run_019_operator_overload, "019_operator_overload");
run_test!(run_020_friend_function, "020_friend_function");
run_test!(run_021_explicit_ctor, "021_explicit_ctor");
run_test!(run_022_mutable_member, "022_mutable_member");
run_test!(run_023_typeid_rtti, "023_typeid_rtti");
run_test!(run_024_template_function, "024_template_function");
run_test!(run_025_template_class, "025_template_class");
run_test!(run_026_template_specialization, "026_template_specialization");
run_test!(run_027_template_instantiation, "027_template_instantiation");
run_test!(run_028_variadic_template, "028_variadic_template");
run_test!(run_029_unique_ptr, "029_unique_ptr");
run_test!(run_030_shared_ptr, "030_shared_ptr");
run_test!(run_031_custom_deleter, "031_custom_deleter");
run_test!(run_032_placement_new, "032_placement_new");
run_test!(run_033_raii_pattern, "033_raii_pattern");
run_test!(run_034_vector_basic, "034_vector_basic");
run_test!(run_035_map_basic, "035_map_basic");
run_test!(run_036_string_basic, "036_string_basic");
run_test!(run_037_array_basic, "037_array_basic");
run_test!(run_038_tuple_basic, "038_tuple_basic");
run_test!(run_039_lambda_basic, "039_lambda_basic");
run_test!(run_040_std_function, "040_std_function");
run_test!(run_041_functional_bind, "041_functional_bind");
run_test!(run_042_exception_basic, "042_exception_basic");
run_test!(run_043_namespace_nested, "043_namespace_nested");
run_test!(run_044_enum_class, "044_enum_class");
run_test!(run_045_union_basic, "045_union_basic");
run_test!(run_046_constexpr_basic, "046_constexpr_basic");
run_test!(run_047_noexcept_basic, "047_noexcept_basic");
run_test!(run_048_summary, "048_summary");
