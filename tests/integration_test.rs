use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use cpp2rust_ffi::{build_project, parser::parse_header_file};

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
