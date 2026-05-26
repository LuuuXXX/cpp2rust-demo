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
        project.main_rs.contains("fn abstract_shape_create_rectangle("),
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
        class.methods.iter().map(|m| &m.rust_name).collect::<Vec<_>>()
    );
    // const methods
    assert!(class.methods[1].is_const, "size() should be const");
    assert!(class.methods[2].is_const, "empty() should be const");
    assert!(class.methods[4].is_const, "at() should be const");
    // return types
    assert_eq!(class.methods[1].return_type.as_deref(), Some("int"));
    assert_eq!(class.methods[2].return_type.as_deref(), Some("bool"));
}
