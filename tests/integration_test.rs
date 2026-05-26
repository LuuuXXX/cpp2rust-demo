use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use cpp2rust_ffi::{
    build_project, parser::parse_header_file, parser::parse_header_str, TodoSummary,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn reset_dir(path: &Path) {
    if path.exists() {
        fs::remove_dir_all(path).unwrap();
    }
    fs::create_dir_all(path).unwrap();
}

#[test]
fn parses_example_headers() {
    let root = repo_root();
    let hello =
        parse_header_file(&root.join("examples/001_hello_world/cpp/hello_world.h")).unwrap();
    let overload =
        parse_header_file(&root.join("examples/002_function_overload/cpp/function_overload.h"))
            .unwrap();
    let class_basic =
        parse_header_file(&root.join("examples/006_class_basic/cpp/class_basic.h")).unwrap();

    assert_eq!(hello.functions.len(), 1);
    assert_eq!(overload.functions.len(), 4);
    assert_eq!(class_basic.classes[0].name, "Counter");
}

#[test]
fn generates_expected_code_for_reference_examples() {
    let root = repo_root();

    let project = build_project(
        &root.join("examples/002_function_overload/cpp"),
        &root.join("target/test-workspaces/function_overload_out"),
        "function_overload",
    )
    .unwrap();

    assert!(project
        .main_rs
        .contains("#[cpp(func = \"int add_int(int, int)\")]"));
    assert!(project
        .main_rs
        .contains("unsafe fn add_strings(a: *const i8, b: *const i8) -> *const i8;"));
    assert!(project.build_rs.contains("function_overload.h"));
    assert!(project.cargo_toml.contains("hicc = { version = \"0.2\" }"));
}

#[test]
fn cli_init_writes_output_project() {
    let root = repo_root();
    let work_dir = root.join("target/test-workspaces/cli_init");
    reset_dir(&work_dir);

    let output_dir = work_dir.join("generated");
    let status = Command::new(env!("CARGO_BIN_EXE_cpp2rust-ffi"))
        .arg("init")
        .arg("--input")
        .arg(root.join("examples/006_class_basic/cpp"))
        .arg("--output")
        .arg(&output_dir)
        .arg("--lib-name")
        .arg("class_basic")
        .status()
        .unwrap();

    assert!(status.success());
    let main_rs = fs::read_to_string(output_dir.join("src/main.rs")).unwrap();
    let cargo_toml = fs::read_to_string(output_dir.join("Cargo.toml")).unwrap();
    let build_rs = fs::read_to_string(output_dir.join("build.rs")).unwrap();

    assert!(main_rs.contains("class Counter;"));
    assert!(main_rs.contains("unsafe fn counter_delete(self_: *mut Counter);"));
    assert!(main_rs.contains("fn counter_new() -> *mut Counter;"));
    assert!(!main_rs.contains("unsafe fn counter_new"));
    assert!(!main_rs.contains("fn counter_get(self_"));
    assert!(cargo_toml.contains("name = \"class_basic\""));
    assert!(build_rs.contains("cc_build.file(cpp_dir.join(\"class_basic.cpp\"));"));
}

/// 025_template_class: template<T> classes must be skipped; concrete IntStack/DoubleStack kept.
#[test]
fn template_class_skips_template_emits_concrete() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/025_template_class/cpp"),
        &root.join("target/test-workspaces/template_class_out"),
        "template_class",
    )
    .unwrap();

    // Template class Stack<T> must NOT appear in import_class!
    assert!(
        !project.main_rs.contains("class Stack {"),
        "template class Stack should be excluded from import_class!"
    );

    // Concrete classes must be present
    assert!(
        project.main_rs.contains("class IntStack {"),
        "IntStack should be in import_class!"
    );
    assert!(
        project.main_rs.contains("class DoubleStack {"),
        "DoubleStack should be in import_class!"
    );

    // Methods should have clean signatures (no inline body leakage)
    assert!(
        project.main_rs.contains("fn size(&self) -> i32;"),
        "size() method should have clean signature"
    );
    assert!(
        project.main_rs.contains("fn empty(&self) -> bool;"),
        "empty() bool return should be mapped correctly"
    );
    assert!(
        project.main_rs.contains("fn push(&mut self, value: i32);"),
        "push(int) should map to i32"
    );
    assert!(
        project.main_rs.contains("fn push(&mut self, value: f64);"),
        "push(double) should map to f64"
    );

    // import_lib! shims for both concrete classes
    assert!(project.main_rs.contains("fn intstack_new()"));
    assert!(project.main_rs.contains("fn doublestack_new()"));
}

/// 013_inheritance_single: base class Animal and derived class Dog should both be parsed.
#[test]
fn inheritance_single_emits_both_classes() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/013_inheritance_single/cpp"),
        &root.join("target/test-workspaces/inheritance_single_out"),
        "inheritance_single",
    )
    .unwrap();

    // Both classes in import_class!
    assert!(
        project.main_rs.contains("class Animal {"),
        "Animal base class should be in import_class!"
    );
    assert!(
        project.main_rs.contains("class Dog {"),
        "Dog derived class should be in import_class!"
    );

    // Constructor/destructor wrappers
    assert!(project.main_rs.contains("fn animal_new("));
    assert!(project.main_rs.contains("unsafe fn animal_delete("));
    assert!(project.main_rs.contains("fn dog_new("));
    assert!(project.main_rs.contains("unsafe fn dog_delete("));

    // const char* params map to *const i8
    assert!(
        project.main_rs.contains("*const i8"),
        "const char* params should map to *const i8"
    );
}

/// 016_virtual_pure: AbstractShape with pure virtual methods should appear in import_class!
#[test]
fn virtual_pure_emits_abstract_class() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/016_virtual_pure/cpp"),
        &root.join("target/test-workspaces/virtual_pure_out"),
        "virtual_pure",
    )
    .unwrap();

    // Abstract class must be present
    assert!(
        project.main_rs.contains("class AbstractShape {"),
        "AbstractShape should be in import_class!"
    );

    // Factory functions in import_lib!
    assert!(
        project.main_rs.contains("fn abstract_shape_create_circle("),
        "factory fn for circle should be present"
    );
    assert!(
        project
            .main_rs
            .contains("fn abstract_shape_create_rectangle("),
        "factory fn for rectangle should be present"
    );
    assert!(
        project.main_rs.contains("unsafe fn abstract_shape_delete("),
        "delete fn should be unsafe"
    );

    // Virtual methods surface in import_class!
    assert!(
        project.main_rs.contains("fn area(&self) -> f64;"),
        "virtual area() method should be in import_class!"
    );
}

/// Batch structural validity: all 47 examples with cpp/ dirs produce valid hicc output.
/// 048_summary has no cpp/ dir and is intentionally excluded.
#[test]
fn all_examples_produce_hicc_output() {
    let root = repo_root();

    // (example_dir_name, lib_name)
    let examples: &[(&str, &str)] = &[
        ("001_hello_world", "hello_world"),
        ("002_function_overload", "function_overload"),
        ("003_default_args", "default_args"),
        ("004_inline_functions", "inline_functions"),
        ("005_variadic_functions", "variadic_functions"),
        ("006_class_basic", "class_basic"),
        ("007_class_constructor", "class_constructor"),
        ("008_class_copy", "class_copy"),
        ("009_class_move", "class_move"),
        ("010_class_static", "class_static"),
        ("011_class_const", "class_const"),
        ("012_class_volatile", "class_volatile"),
        ("013_inheritance_single", "inheritance_single"),
        ("014_inheritance_multiple", "inheritance_multiple"),
        ("015_virtual_basic", "virtual_basic"),
        ("016_virtual_pure", "virtual_pure"),
        ("017_virtual_override", "virtual_override"),
        ("018_virtual_diamond", "virtual_diamond"),
        ("019_operator_overload", "operator_overload"),
        ("020_friend_function", "friend_function"),
        ("021_explicit_ctor", "explicit_ctor"),
        ("022_mutable_member", "mutable_member"),
        ("023_typeid_rtti", "typeid_rtti"),
        ("024_template_function", "template_function"),
        ("025_template_class", "template_class"),
        ("026_template_specialization", "template_specialization"),
        ("027_template_instantiation", "template_instantiation"),
        ("028_variadic_template", "variadic_template"),
        ("029_unique_ptr", "unique_ptr"),
        ("030_shared_ptr", "shared_ptr"),
        ("031_custom_deleter", "custom_deleter"),
        ("032_placement_new", "placement_new"),
        ("033_raii_pattern", "raii_pattern"),
        ("034_vector_basic", "vector_basic"),
        ("035_map_basic", "map_basic"),
        ("036_string_basic", "string_basic"),
        ("037_array_basic", "array_basic"),
        ("038_tuple_basic", "tuple_basic"),
        ("039_lambda_basic", "lambda_basic"),
        ("040_std_function", "std_function"),
        ("041_functional_bind", "functional_bind"),
        ("042_exception_basic", "exception_basic"),
        ("043_namespace_nested", "namespace_nested"),
        ("044_enum_class", "enum_class"),
        ("045_union_basic", "union_basic"),
        ("046_constexpr_basic", "constexpr_basic"),
        ("047_noexcept_basic", "noexcept_basic"),
    ];

    for (dir, lib_name) in examples {
        let project = build_project(
            &root.join(format!("examples/{dir}/cpp")),
            &root.join(format!("target/test-workspaces/batch/{dir}")),
            lib_name,
        )
        .unwrap_or_else(|e| panic!("{dir}: build_project failed: {e}"));

        assert!(
            project.main_rs.contains("hicc::cpp! {"),
            "{dir}: output must begin with hicc::cpp! block"
        );
        assert!(
            project.main_rs.contains("hicc::import_lib! {")
                || project.main_rs.contains("hicc::import_class! {"),
            "{dir}: output must contain at least one hicc import macro block"
        );
        assert!(
            !project.main_rs.is_empty(),
            "{dir}: main_rs must be non-empty"
        );
    }
}

/// 007_class_constructor: Point class with multiple ctors; methods include getX/getY/getMagnitude.
#[test]
fn class_constructor_emits_point_methods() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/007_class_constructor/cpp"),
        &root.join("target/test-workspaces/class_constructor_out"),
        "class_constructor",
    )
    .unwrap();

    assert!(
        project.main_rs.contains("class Point {"),
        "Point class should be in import_class!"
    );
    assert!(
        project.main_rs.contains("fn get_x(&self) -> i32;"),
        "getX() method should map to get_x"
    );
    assert!(
        project.main_rs.contains("fn get_y(&self) -> i32;"),
        "getY() method should map to get_y"
    );
    assert!(
        project.main_rs.contains("fn get_magnitude(&self) -> f64;"),
        "getMagnitude() double return should map to f64"
    );
    // Multiple constructor variants
    assert!(
        project.main_rs.contains("fn point_new") || project.main_rs.contains("fn point_new_"),
        "point_new constructor wrapper should be present"
    );
    assert!(
        project.main_rs.contains("unsafe fn point_delete("),
        "point_delete should be unsafe"
    );
}

/// 008_class_copy and 009_class_move: copy/move classes emit Buffer/UniqueVector with correct methods.
#[test]
fn class_copy_and_move_emit_correct_classes() {
    let root = repo_root();

    // 008: Buffer with copy semantics
    let copy_project = build_project(
        &root.join("examples/008_class_copy/cpp"),
        &root.join("target/test-workspaces/class_copy_out"),
        "class_copy",
    )
    .unwrap();
    assert!(
        copy_project.main_rs.contains("class Buffer {"),
        "Buffer class should be in import_class!"
    );
    assert!(
        copy_project.main_rs.contains("fn get_size(&self) -> i32;"),
        "getSize() should appear in import_class!"
    );
    assert!(
        copy_project.main_rs.contains("fn buffer_new_copy("),
        "buffer_new_copy (copy ctor wrapper) should be present"
    );

    // 009: UniqueVector with move semantics
    let move_project = build_project(
        &root.join("examples/009_class_move/cpp"),
        &root.join("target/test-workspaces/class_move_out"),
        "class_move",
    )
    .unwrap();
    assert!(
        move_project.main_rs.contains("class UniqueVector {"),
        "UniqueVector class should be in import_class!"
    );
    assert!(
        move_project.main_rs.contains("fn get_size(&self) -> i32;"),
        "getSize() method should be present"
    );
}

/// 010_class_static: static member functions appear in import_lib! as free functions.
#[test]
fn class_static_emits_static_wrappers_in_import_lib() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/010_class_static/cpp"),
        &root.join("target/test-workspaces/class_static_out"),
        "class_static",
    )
    .unwrap();

    // Non-static methods in import_class!
    assert!(
        project.main_rs.contains("class Counter {"),
        "Counter should be in import_class!"
    );
    assert!(
        project.main_rs.contains("fn get_value(&self) -> i32;"),
        "getValue() should be in import_class!"
    );

    // Static member wrappers in import_lib!
    assert!(
        project.main_rs.contains("fn counter_get_instance_count()"),
        "static getInstanceCount() wrapper should be in import_lib!"
    );
    assert!(
        project
            .main_rs
            .contains("fn counter_reset_instance_count()"),
        "static resetInstanceCount() wrapper should be in import_lib!"
    );
}

/// 017_virtual_override: Base and Derived both appear; override method area() in both classes.
#[test]
fn virtual_override_emits_base_and_derived_with_area() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/017_virtual_override/cpp"),
        &root.join("target/test-workspaces/virtual_override_out"),
        "virtual_override",
    )
    .unwrap();

    assert!(
        project.main_rs.contains("class Base {"),
        "Base should be in import_class!"
    );
    assert!(
        project.main_rs.contains("class Derived {"),
        "Derived should be in import_class!"
    );
    // area() is virtual-overridden; both classes should expose it
    assert!(
        project.main_rs.matches("fn area(&self) -> f64;").count() >= 2,
        "area() should appear in both Base and Derived import_class! blocks"
    );
    assert!(
        project.main_rs.contains("fn derived_new("),
        "derived_new factory should be in import_lib!"
    );
}

/// 018_virtual_diamond: diamond inheritance — A, B, C, D all emitted.
#[test]
fn virtual_diamond_emits_all_four_classes() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/018_virtual_diamond/cpp"),
        &root.join("target/test-workspaces/virtual_diamond_out"),
        "virtual_diamond",
    )
    .unwrap();

    for cls in &["A", "B", "C", "D"] {
        assert!(
            project.main_rs.contains(&format!("class {cls} {{")),
            "{cls} should be in import_class! for diamond inheritance"
        );
    }
    assert!(
        project.main_rs.contains("fn get_a_value(&self) -> i32;"),
        "getAValue() from base A should surface in import_class!"
    );
}

/// 019_operator_overload: Number class emits import_class! methods + named operator shims in import_lib!.
#[test]
fn operator_overload_emits_number_class_and_shims() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/019_operator_overload/cpp"),
        &root.join("target/test-workspaces/operator_overload_out"),
        "operator_overload",
    )
    .unwrap();

    // Class binding
    assert!(
        project.main_rs.contains("class Number {"),
        "Number should be in import_class!"
    );
    assert!(
        project.main_rs.contains("fn get_value(&self) -> i32;"),
        "getValue() method should appear"
    );

    // Named shims for arithmetic operators
    assert!(
        project.main_rs.contains("fn number_add("),
        "operator+ shim number_add should be in import_lib!"
    );
    assert!(
        project.main_rs.contains("fn number_sub("),
        "operator- shim number_sub should be in import_lib!"
    );
    assert!(
        project.main_rs.contains("fn number_mul("),
        "operator* shim number_mul should be in import_lib!"
    );

    // Compound-assignment shims
    assert!(
        project.main_rs.contains("fn number_add_assign("),
        "operator+= shim should be present"
    );

    // Constructor/destructor
    assert!(project.main_rs.contains("fn number_new("));
    assert!(project.main_rs.contains("unsafe fn number_delete("));

    // cpp2rust-todo[OP] comments must appear after operator shim entries
    assert!(
        project
            .main_rs
            .contains("// cpp2rust-todo[OP]: Consider implementing std::ops traits for Number"),
        "operator shim entries should carry a cpp2rust-todo[OP] inline comment"
    );
}

/// 020_friend_function: friend functions appear as free functions in import_lib!.
#[test]
fn friend_function_emits_free_functions_in_import_lib() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/020_friend_function/cpp"),
        &root.join("target/test-workspaces/friend_function_out"),
        "friend_function",
    )
    .unwrap();

    // Class binding exists
    assert!(
        project.main_rs.contains("class MyClass {"),
        "MyClass should be in import_class!"
    );

    // Friend functions as free wrappers
    assert!(
        project.main_rs.contains("fn friend_function_get_sum("),
        "friend getSum should be emitted as free fn"
    );
    assert!(
        project.main_rs.contains("fn friend_function_get_product("),
        "friend getProduct should be emitted as free fn"
    );
    assert!(
        project.main_rs.contains("fn friend_function_compare("),
        "friend compare should be emitted as free fn"
    );
}

/// 039_lambda_basic: lambda wrappers and stateful lambda classes appear in import_lib!.
#[test]
fn lambda_basic_emits_state_lambda_and_fn_ptr_type() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/039_lambda_basic/cpp"),
        &root.join("target/test-workspaces/lambda_basic_out"),
        "lambda_basic",
    )
    .unwrap();

    // Stateful lambda opaque classes declared
    assert!(
        project.main_rs.contains("class StateLambda;"),
        "StateLambda opaque class should be declared"
    );
    assert!(
        project.main_rs.contains("class LambdaWrapper;"),
        "LambdaWrapper opaque class should be declared"
    );

    // Stateless lambda: fn-pointer-based apply_operation
    assert!(
        project.main_rs.contains("fn apply_operation("),
        "apply_operation (fn-ptr lambda) should be in import_lib!"
    );

    // Stateful lambda factory and methods
    assert!(
        project.main_rs.contains("fn state_lambda_new("),
        "state_lambda_new factory should be present"
    );
    assert!(
        project.main_rs.contains("fn state_lambda_apply("),
        "state_lambda_apply should be present"
    );
    assert!(
        project.main_rs.contains("fn make_add_lambda("),
        "make_add_lambda factory should be present"
    );
}

/// 044_enum_class: enum-backed OperationResult class; enum types map via underlying int/uint.
#[test]
fn enum_class_emits_operation_result_with_typed_methods() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/044_enum_class/cpp"),
        &root.join("target/test-workspaces/enum_class_out"),
        "enum_class",
    )
    .unwrap();

    assert!(
        project.main_rs.contains("class OperationResult {"),
        "OperationResult should be in import_class!"
    );

    // Enum-typed setters/getters map to underlying primitive types
    assert!(
        project
            .main_rs
            .contains("fn set_error(&mut self, code: i32);"),
        "set_error(int) should map to i32"
    );
    assert!(
        project.main_rs.contains("fn get_error(&self) -> i32;"),
        "get_error() -> int should map to i32"
    );
    assert!(
        project.main_rs.contains("fn get_state(&self) -> u8;"),
        "get_state() -> unsigned char should map to u8"
    );
    assert!(
        project.main_rs.contains("fn get_flags(&self) -> u32;"),
        "get_flags() -> unsigned int should map to u32"
    );

    // Opaque enum class destructors in import_lib!
    assert!(
        project.main_rs.contains("unsafe fn error_code_delete("),
        "ErrorCode destructor should be present"
    );
    assert!(
        project.main_rs.contains("fn operation_result_new()"),
        "OperationResult constructor wrapper should be present"
    );
}

/// 023_typeid_rtti: Shape hierarchy with integer-enum type dispatch and virtual factories.
/// Verifies Phase 7 (RTTI enum injection) — factory shims and area/type accessors all present.
#[test]
fn typeid_rtti_emits_shape_factories_and_type_accessors() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/023_typeid_rtti/cpp"),
        &root.join("target/test-workspaces/typeid_rtti_out"),
        "typeid_rtti",
    )
    .unwrap();

    // Factory shims for each concrete shape (from C-side header)
    assert!(
        project.main_rs.contains("fn shape_new_circle("),
        "shape_new_circle factory should be in import_lib!"
    );
    assert!(
        project.main_rs.contains("fn shape_new_rectangle("),
        "shape_new_rectangle factory should be in import_lib!"
    );
    assert!(
        project.main_rs.contains("fn shape_new_triangle("),
        "shape_new_triangle factory should be in import_lib!"
    );

    // Destructor
    assert!(
        project.main_rs.contains("unsafe fn shape_delete("),
        "shape_delete destructor should be present"
    );

    // Type-tag and area virtual methods appear in import_class! (integer-enum strategy, not mangled names)
    assert!(
        project.main_rs.contains("fn get_type(&self) -> i32;"),
        "getType() should map to fn get_type(&self) -> i32 in import_class!"
    );
    assert!(
        project.main_rs.contains("fn area(&self) -> f64;"),
        "area() should map to fn area(&self) -> f64 in import_class!"
    );

    // Base class appears in both import_class! (with methods) and import_lib! (forward decl)
    assert!(
        project.main_rs.contains("class Shape {"),
        "Shape should appear in import_class! block with methods"
    );
    assert!(
        project.main_rs.contains("class Shape;"),
        "Shape forward declaration should appear in import_lib! block"
    );
}

/// 041_functional_bind: std::bind pattern — opaque class wrappers (Adder, Multiplier, StringProcessor)
/// plus directly-bound free functions (add_five, add_ten).
/// Verifies that std::bind is fully supported via the class-wrapper strategy (v4 upgrade from ⚠️ to ✅).
#[test]
fn functional_bind_emits_class_wrappers_and_bound_free_fns() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/041_functional_bind/cpp"),
        &root.join("target/test-workspaces/functional_bind_out"),
        "functional_bind",
    )
    .unwrap();

    // Opaque class forward declarations in import_lib!
    assert!(
        project.main_rs.contains("class Adder;"),
        "Adder opaque class should be declared"
    );
    assert!(
        project.main_rs.contains("class Multiplier;"),
        "Multiplier opaque class should be declared"
    );
    assert!(
        project.main_rs.contains("class StringProcessor;"),
        "StringProcessor opaque class should be declared"
    );

    // Adder factory / method / destructor
    assert!(
        project.main_rs.contains("fn adder_new("),
        "adder_new constructor shim should be in import_lib!"
    );
    assert!(
        project.main_rs.contains("unsafe fn adder_delete("),
        "adder_delete destructor shim should be in import_lib!"
    );
    assert!(
        project.main_rs.contains("fn adder_add("),
        "adder_add method shim should be in import_lib!"
    );

    // Multiplier factory / method
    assert!(
        project.main_rs.contains("fn multiplier_new("),
        "multiplier_new constructor shim should be in import_lib!"
    );
    assert!(
        project
            .main_rs
            .contains("fn multiply(self_: *mut Multiplier"),
        "multiply method shim should be in import_lib!"
    );

    // Directly-bound free functions (pre-applied std::bind)
    assert!(
        project.main_rs.contains("fn add_five("),
        "add_five bound free fn should be in import_lib!"
    );
    assert!(
        project.main_rs.contains("fn add_ten("),
        "add_ten bound free fn should be in import_lib!"
    );
}

/// 025_template_class inline body stripping: parse header with inline method defs.
#[test]
fn inline_method_bodies_stripped_in_class_parse() {
    let header = r#"
        class MyVec {
        public:
            MyVec() = default;
            int size() const { return static_cast<int>(data_.size()); }
            bool empty() const { return data_.empty(); }
            void push_back(int val) { data_.push_back(val); }
            int at(int i) const { return data_.at(i); }
        };
    "#;
    let parsed = parse_header_str("myvec.h", header).unwrap();
    let class = &parsed.classes[0];
    assert_eq!(class.name, "MyVec");
    // constructor + 4 methods (size, empty, push_back, at)
    assert_eq!(
        class.methods.len(),
        5,
        "expected constructor + 4 inline methods, got {:?}",
        class
            .methods
            .iter()
            .map(|m| &m.rust_name)
            .collect::<Vec<_>>()
    );
    // const methods
    assert!(class.methods[1].is_const, "size() should be const");
    assert!(class.methods[2].is_const, "empty() should be const");
    assert!(class.methods[4].is_const, "at() should be const");
    // return types
    assert_eq!(class.methods[1].return_type.as_deref(), Some("int"));
    assert_eq!(class.methods[2].return_type.as_deref(), Some("bool"));
}

/// 028_variadic_template: fixed-arity expansion produces sum_2, sum_3, and sum_double_2.
#[test]
fn variadic_template_emits_fixed_arity_functions() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/028_variadic_template/cpp"),
        &root.join("target/test-workspaces/variadic_template_out"),
        "variadic_template",
    )
    .unwrap();

    // Fixed-arity int versions must be present
    assert!(
        project.main_rs.contains("fn sum_2("),
        "sum_2 fixed-arity shim should be in import_lib!"
    );
    assert!(
        project.main_rs.contains("fn sum_3("),
        "sum_3 fixed-arity shim should be in import_lib!"
    );

    // Double overload must be present
    assert!(
        project.main_rs.contains("fn sum_double_2("),
        "sum_double_2 double-typed shim should be in import_lib!"
    );

    // Return types must be mapped correctly
    assert!(
        project.main_rs.contains("-> i32"),
        "int return type should map to i32"
    );
    assert!(
        project.main_rs.contains("-> f64"),
        "double return type should map to f64"
    );
}

/// 020_friend_function: friend functions carry cpp2rust-todo[FR] inline comments.
#[test]
fn friend_function_emits_fr_todo_comments() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/020_friend_function/cpp"),
        &root.join("target/test-workspaces/friend_function_fr_out"),
        "friend_function",
    )
    .unwrap();

    assert!(
        project
            .main_rs
            .contains("// cpp2rust-todo[FR]: Friend function"),
        "friend functions should carry a cpp2rust-todo[FR] inline comment"
    );
    // TodoSummary should count the FR entries
    assert!(
        project.todo_summary.fr_count >= 1,
        "todo_summary.fr_count should be >= 1 for friend_function example"
    );
}

/// 039_lambda_basic: fn-pointer parameters carry cpp2rust-todo[LM] inline comments.
#[test]
fn lambda_basic_emits_lm_todo_comments() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/039_lambda_basic/cpp"),
        &root.join("target/test-workspaces/lambda_basic_lm_out"),
        "lambda_basic",
    )
    .unwrap();

    assert!(
        project
            .main_rs
            .contains("// cpp2rust-todo[LM]: fn-pointer / lambda parameter"),
        "functions with fn-ptr params should carry a cpp2rust-todo[LM] inline comment"
    );
    assert!(
        project.todo_summary.lm_count >= 1,
        "todo_summary.lm_count should be >= 1 for lambda_basic example"
    );
}

/// 028_variadic_template: arity-expanded functions carry cpp2rust-todo[VA] inline comments.
#[test]
fn variadic_template_emits_va_todo_comments() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/028_variadic_template/cpp"),
        &root.join("target/test-workspaces/variadic_template_va_out"),
        "variadic_template",
    )
    .unwrap();

    assert!(
        project
            .main_rs
            .contains("// cpp2rust-todo[VA]: Variadic template fixed-arity expansion"),
        "arity-expanded functions should carry a cpp2rust-todo[VA] inline comment"
    );
    assert!(
        project.todo_summary.va_count >= 1,
        "todo_summary.va_count should be >= 1 for variadic_template example"
    );
}

/// TodoSummary counts are consistent with grep on the generated source.
#[test]
fn todo_summary_counts_match_source() {
    let root = repo_root();
    // Use operator_overload which we know produces [OP] entries
    let project = build_project(
        &root.join("examples/019_operator_overload/cpp"),
        &root.join("target/test-workspaces/todo_summary_check"),
        "operator_overload",
    )
    .unwrap();

    let src = &project.main_rs;
    let summary = &project.todo_summary;

    assert_eq!(
        summary.op_count,
        src.matches("cpp2rust-todo[OP]").count(),
        "op_count should match grep count"
    );
    assert_eq!(
        summary.fr_count,
        src.matches("cpp2rust-todo[FR]").count(),
        "fr_count should match grep count"
    );
    assert_eq!(
        summary.lm_count,
        src.matches("cpp2rust-todo[LM]").count(),
        "lm_count should match grep count"
    );
    assert_eq!(
        summary.va_count,
        src.matches("cpp2rust-todo[VA]").count(),
        "va_count should match grep count"
    );
    assert_eq!(
        summary.total(),
        summary.op_count
            + summary.fr_count
            + summary.lm_count
            + summary.rtti_count
            + summary.va_count,
        "total() should sum all counts"
    );
}

/// TodoSummary::default() yields all-zero counts.
#[test]
fn todo_summary_default_is_zero() {
    let s = TodoSummary::default();
    assert_eq!(s.total(), 0);
}

/// 023_typeid_rtti: classes with integer type-discriminator methods carry [RTTI] inline TODO.
/// Verifies v4 §5.1 — [RTTI] tag must appear alongside classes that use getType() patterns.
#[test]
fn typeid_rtti_emits_rtti_todo_tag() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/023_typeid_rtti/cpp"),
        &root.join("target/test-workspaces/typeid_rtti_rtti_tag"),
        "typeid_rtti",
    )
    .unwrap();

    assert!(
        project.main_rs.contains("cpp2rust-todo[RTTI]"),
        "RTTI type-discriminator classes should carry a cpp2rust-todo[RTTI] inline comment"
    );
    // 023 has 4 classes (Shape, Circle, Rectangle, Triangle) all with getType() → 4 tags.
    assert_eq!(
        project.todo_summary.rtti_count, 4,
        "todo_summary.rtti_count should be 4 for typeid_rtti example (one per class with getType)"
    );
}

/// 023_typeid_rtti: rtti_count in TodoSummary matches grep count in generated source.
#[test]
fn todo_summary_rtti_count_matches_source() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/023_typeid_rtti/cpp"),
        &root.join("target/test-workspaces/typeid_rtti_rtti_count"),
        "typeid_rtti",
    )
    .unwrap();

    let src = &project.main_rs;
    let summary = &project.todo_summary;
    assert_eq!(
        summary.rtti_count,
        src.matches("cpp2rust-todo[RTTI]").count(),
        "rtti_count should match grep count in generated source"
    );
}

/// Non-RTTI examples must not carry a spurious [RTTI] tag.
/// Critically, 045_union_basic has Variant::get_type() but NO get_type_name() — must NOT fire.
#[test]
fn non_rtti_examples_have_no_rtti_tag() {
    let root = repo_root();
    for (dir, lib) in [
        ("examples/001_hello_world/cpp", "hello_world"),
        ("examples/006_class_basic/cpp", "class_basic"),
        ("examples/019_operator_overload/cpp", "operator_overload"),
        ("examples/013_inheritance_single/cpp", "inheritance_single"),
        // 045: Variant has get_type()→int but NOT get_type_name()→const char* — must be no false positive
        ("examples/045_union_basic/cpp", "union_basic"),
    ] {
        let project = build_project(
            &root.join(dir),
            &root.join(format!("target/test-workspaces/no_rtti/{}", lib)),
            lib,
        )
        .unwrap();
        assert_eq!(
            project.todo_summary.rtti_count, 0,
            "{dir} should have rtti_count == 0 (no RTTI pattern)"
        );
    }
}

/// 028_variadic_template: [VA] fires on arity-expansion groups; constructor overloads must not fire.
#[test]
fn variadic_template_no_false_positive_on_constructors() {
    let root = repo_root();

    // 036_string_basic has StringImpl with 3 constructors → _new, _new_1, _new_2.
    // These must NOT be tagged [VA]; they are constructor overloads, not template expansions.
    let project_036 = build_project(
        &root.join("examples/036_string_basic/cpp"),
        &root.join("target/test-workspaces/no_va_036"),
        "string_basic",
    )
    .unwrap();
    assert_eq!(
        project_036.todo_summary.va_count, 0,
        "036_string_basic constructor overloads must not be tagged [VA]"
    );

    // 005_variadic_functions has sum_3, sum_5 as C-variadic wrappers for sum(int, ...).
    // The base `sum` already exists in the function list → must NOT be tagged [VA].
    let project_005 = build_project(
        &root.join("examples/005_variadic_functions/cpp"),
        &root.join("target/test-workspaces/no_va_005"),
        "variadic_functions",
    )
    .unwrap();
    assert_eq!(
        project_005.todo_summary.va_count, 0,
        "005_variadic_functions C-variadic wrappers (sum_3, sum_5) must not be tagged [VA]"
    );

    // 028_variadic_template must still have [VA] tags (genuine variadic template expansions).
    let project_028 = build_project(
        &root.join("examples/028_variadic_template/cpp"),
        &root.join("target/test-workspaces/va_028"),
        "variadic_template",
    )
    .unwrap();
    assert!(
        project_028.todo_summary.va_count >= 2,
        "028_variadic_template should have at least 2 [VA] tags (genuine variadic expansions)"
    );
}

/// 039_lambda_basic: fn-ptr typedef type aliases are emitted inside import_lib!.
/// Verifies the fix for v4 §4.4.1 — `type IntBinaryOp = extern "C" fn(i32, i32) -> i32;`
/// must appear so that import_lib! function signatures referencing IntBinaryOp compile.
#[test]
fn lambda_basic_emits_fn_ptr_typedef_alias() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/039_lambda_basic/cpp"),
        &root.join("target/test-workspaces/lambda_basic_typedef_out"),
        "lambda_basic",
    )
    .unwrap();

    // The type alias must be present so that `op: IntBinaryOp` in import_lib! is a valid Rust type.
    assert!(
        project
            .main_rs
            .contains("type IntBinaryOp = extern \"C\" fn(i32, i32) -> i32;"),
        "IntBinaryOp typedef should be emitted as a Rust type alias inside import_lib!"
    );
}

/// 039_lambda_basic: fn main() demo must NOT call apply_operation with literal `0` for
/// the fn-pointer parameter — functions with fn-ptr params are skipped in the demo.
#[test]
fn lambda_basic_demo_skips_fn_ptr_functions() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/039_lambda_basic/cpp"),
        &root.join("target/test-workspaces/lambda_basic_demo_out"),
        "lambda_basic",
    )
    .unwrap();

    // apply_operation has an IntBinaryOp (fn-ptr) param; the demo must not call it.
    assert!(
        !project.main_rs.contains("apply_operation(0, 0, 0)"),
        "fn main() must not call apply_operation with literal 0 for a function-pointer param"
    );
    assert!(
        !project.main_rs.contains("apply_twice(0, 0)"),
        "fn main() must not call apply_twice with literal 0 for a function-pointer param"
    );
}

/// 031_custom_deleter: `= delete` copy constructor must NOT generate a shim.
/// Verifies v4 §9 — generated code must be valid Rust.  The deleted copy constructor
/// `FileHandle(const FileHandle&) = delete` must be skipped; a shim for it would contain
/// an invalid Rust identifier (`file_handle&`) and call an inaccessible C++ function.
#[test]
fn deleted_constructors_produce_no_shim() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/031_custom_deleter/cpp"),
        &root.join("target/test-workspaces/deleted_ctor_031"),
        "custom_deleter",
    )
    .unwrap();

    // The deleted copy constructor would have been named `file_handle_new_1` if not skipped.
    // Its presence in import_lib! as `fn file_handle_new_1(file_handle&: const)` would be
    // invalid Rust — the parameter name contains `&`.
    assert!(
        !project.main_rs.contains("file_handle&: const"),
        "deleted copy constructor must not generate an invalid Rust binding"
    );
}

/// 031_custom_deleter: `= delete` methods must not appear in the hicc::cpp! shim body.
/// The shim body `new FileHandle(FileHandle&)` is invalid C++ (calling a deleted constructor).
#[test]
fn deleted_constructor_shim_body_absent() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/031_custom_deleter/cpp"),
        &root.join("target/test-workspaces/deleted_ctor_body_031"),
        "custom_deleter",
    )
    .unwrap();

    // The old broken shim body was `new FileHandle(FileHandle&)` — invalid C++.
    assert!(
        !project.main_rs.contains("new FileHandle(FileHandle&)"),
        "deleted copy constructor must not appear in the hicc::cpp! shim body"
    );
}

/// parse_param correctly handles unnamed reference parameters (e.g. `const Foo&`).
/// When a parameter declaration is a reference type without an explicit name, the parser
/// must NOT misidentify the type suffix as the parameter name (yielding `foo&: const`).
/// It should fall back to an anonymous `arg0` name with the full type.
#[test]
fn unnamed_reference_param_gets_anonymous_name() {
    use cpp2rust_ffi::parser::parse_header_str;
    // Simulate a class with a constructor taking an unnamed const reference (like `= default`)
    let header_src = r#"
        class Wrapper {
        public:
            Wrapper(const Wrapper&);
        };
        Wrapper* wrapper_new_copy(const Wrapper* other);
    "#;
    let parsed = parse_header_str("wrapper.h", header_src).unwrap();
    let class = &parsed.classes[0];
    // The constructor has one param: `const Wrapper&`
    let ctor = class
        .methods
        .iter()
        .find(|m| matches!(m.kind, cpp2rust_ffi::ir::MethodKind::Constructor))
        .expect("constructor must be present");
    let param = &ctor.params[0];
    // Parameter name must NOT contain `&` (it was previously mis-parsed as "Wrapper&")
    assert!(
        !param.name.contains('&'),
        "parameter name must not contain '&': got '{}'",
        param.name
    );
    // The type should contain the full reference type
    assert!(
        param.cpp_type.contains("Wrapper"),
        "parameter cpp_type must contain 'Wrapper': got '{}'",
        param.cpp_type
    );
}

/// 031_custom_deleter: `typedef void (*FileDeleter)(struct FileHandle*)` must generate
/// `type FileDeleter = extern "C" fn(*mut FileHandle)` — NOT `extern "C" fn(struct)`.
/// Verifies Bug D fix: `parse_typedefs` must not mis-split `struct Foo*` arg as type="struct".
#[test]
fn typedef_struct_ptr_arg_maps_to_correct_rust_type() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/031_custom_deleter/cpp"),
        &root.join("target/test-workspaces/typedef_struct_ptr_031"),
        "custom_deleter",
    )
    .unwrap();

    assert!(
        project
            .main_rs
            .contains("type FileDeleter = extern \"C\" fn(*mut FileHandle)"),
        "FileDeleter typedef should map to `extern \"C\" fn(*mut FileHandle)`, not `fn(struct)`"
    );
}

/// 031_custom_deleter: `FileHandle&&` (rvalue reference) in a constructor shim must map to
/// `*mut FileHandle` — NOT `*mut *mut FileHandle`.
/// Verifies Bug E fix: `&&` suffix must be handled before single `&` in typemap.
#[test]
fn rvalue_ref_param_maps_to_single_pointer() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/031_custom_deleter/cpp"),
        &root.join("target/test-workspaces/rvalue_ref_031"),
        "custom_deleter",
    )
    .unwrap();

    assert!(
        !project.main_rs.contains("*mut *mut"),
        "FileHandle&& must map to *mut FileHandle, not *mut *mut FileHandle"
    );
    assert!(
        project
            .main_rs
            .contains("fn file_handle_new_1(arg0: *mut FileHandle)"),
        "file_handle_new_1 should take *mut FileHandle, not *mut *mut FileHandle"
    );
}

/// 031_custom_deleter: fn main() must NOT call `file_handle_new` (which takes a FileDeleter
/// fn-ptr parameter) with literal `0` — that would fail to compile.
/// Verifies Bug F fix: constructors with fn-ptr typedef params are skipped in the demo.
#[test]
fn demo_skips_constructor_with_fn_ptr_typedef_param() {
    let root = repo_root();
    let project = build_project(
        &root.join("examples/031_custom_deleter/cpp"),
        &root.join("target/test-workspaces/ctor_fn_ptr_skip_031"),
        "custom_deleter",
    )
    .unwrap();

    assert!(
        !project
            .main_rs
            .contains("file_handle_new(std::ptr::null(), std::ptr::null(), 0)"),
        "fn main() must not call file_handle_new with literal 0 for a FileDeleter fn-ptr param"
    );
}

/// typemap: rvalue reference `T&&` must map to `*mut T` (same as `T&`), not `*mut *mut T`.
#[test]
fn typemap_rvalue_ref_maps_same_as_lvalue_ref() {
    use cpp2rust_ffi::typemap::map_cpp_type_to_rust;
    assert_eq!(map_cpp_type_to_rust("FileHandle&&"), "*mut FileHandle");
    assert_eq!(map_cpp_type_to_rust("FileHandle&"), "*mut FileHandle");
    assert_eq!(map_cpp_type_to_rust("const Foo&&"), "*const Foo");
}
