use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn test_dir(name: &str) -> PathBuf {
    let root = repo_root()
        .join("target/test-work/integration")
        .join(format!("{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn init_processes_simple_cpp_project() {
    let temp = test_dir("simple-project");
    fs::write(
        temp.join("main.cpp"),
        r#"int add(int a, int b) { return a + b; }"#,
    )
    .unwrap();
    fs::write(
        temp.join("Makefile"),
        "CXX=g++\nall:\n\t$(CXX) -c -std=c++17 main.cpp -o main.o\nclean:\n\trm -f main.o\n",
    )
    .unwrap();

    Command::cargo_bin("cpp2rust-demo")
        .unwrap()
        .current_dir(&temp)
        .args(["init", "--feature", "simple", "--", "sh", "-c", "make clean && make"]) 
        .assert()
        .success()
        .stdout(predicate::str::contains("generated 1 translation unit"));

    assert!(temp.join(".cpp2rust/simple/ast/main.cpp.json").exists());
    assert!(temp.join(".cpp2rust/simple/rust/src/main/mod.rs").exists());
    assert!(temp.join(".cpp2rust/simple/meta/selected_files.json").exists());
}

#[test]
fn merge_produces_expected_output_structure() {
    let example = repo_root().join("examples/01-basic-types");
    let feature_root = example.join(".cpp2rust/itest_merge");
    if feature_root.exists() {
        fs::remove_dir_all(&feature_root).unwrap();
    }

    Command::cargo_bin("cpp2rust-demo")
        .unwrap()
        .current_dir(&example)
        .args(["init", "--feature", "itest_merge", "--", "sh", "-c", "make clean && make"]) 
        .assert()
        .success();

    Command::cargo_bin("cpp2rust-demo")
        .unwrap()
        .current_dir(&example)
        .args(["merge", "--feature", "itest_merge"]) 
        .assert()
        .success()
        .stdout(predicate::str::contains("merged"));

    assert!(feature_root.join("rust/src.1").exists());
    assert!(feature_root.join("rust/src.2/lib.rs").exists());
    let metadata = fs::symlink_metadata(feature_root.join("rust/src")).unwrap();
    assert!(metadata.file_type().is_symlink());
}

#[test]
fn init_succeeds_for_basic_types_example() {
    let example = repo_root().join("examples/01-basic-types");
    let feature_root = example.join(".cpp2rust/itest_basic");
    if feature_root.exists() {
        fs::remove_dir_all(&feature_root).unwrap();
    }

    Command::cargo_bin("cpp2rust-demo")
        .unwrap()
        .current_dir(&example)
        .args(["init", "--feature", "itest_basic", "--", "sh", "-c", "make clean && make"]) 
        .assert()
        .success();

    assert!(feature_root.join("rust/Cargo.toml").exists());
    assert!(feature_root.join("meta/init-interface-report.md").exists());
}
