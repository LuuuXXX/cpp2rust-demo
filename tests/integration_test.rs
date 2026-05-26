use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use cpp2rust_ffi::{build_project, parser::parse_header_file, parser::parse_header_str};

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
        project.main_rs.contains("fn multiply(self_: *mut Multiplier"),
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
