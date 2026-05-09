// Integration tests for the cpp2rust-demo CLI.
//
// These tests run the compiled binary against real C++ headers (using the
// `clang` binary on the host) and verify the generated output.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

// Helper: get the binary path.
fn bin() -> Command {
    Command::cargo_bin("cpp2rust-demo").unwrap()
}

// Helper: write a C++ header to a temporary file and return its path.
fn write_header(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path
}

// ---------------------------------------------------------------------------
// Basic CLI sanity
// ---------------------------------------------------------------------------

#[test]
fn help_flag_exits_zero() {
    bin().arg("--help").assert().success();
}

#[test]
fn version_flag_exits_zero() {
    bin().arg("--version").assert().success();
}

#[test]
fn init_without_link_fails() {
    let tmp = TempDir::new().unwrap();
    let h = write_header(&tmp, "test.hpp", "void foo();");
    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            h.to_str().unwrap(),
        ])
        .assert()
        .failure();
}

#[test]
fn init_nonexistent_header_fails() {
    let tmp = TempDir::new().unwrap();
    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "mylib",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            "does_not_exist.hpp",
        ])
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// init command
// ---------------------------------------------------------------------------

#[test]
fn init_simple_free_functions() {
    let tmp = TempDir::new().unwrap();
    let h = write_header(
        &tmp,
        "mylib.hpp",
        r#"
        int add(int a, int b);
        double scale(double x, double factor);
        "#,
    );

    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "mylib",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            h.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ cpp2rust-demo init completed"));

    // Check that the generated FFI file exists.
    let ffi = tmp
        .path()
        .join(".cpp2rust/default/rust/src/ffi_mylib.rs");
    assert!(ffi.exists(), "ffi_mylib.rs should exist");

    let content = std::fs::read_to_string(&ffi).unwrap();
    assert!(content.contains("import_lib!"));
    assert!(content.contains("link_name = \"mylib\""));
    assert!(content.contains("fn add(a: i32, b: i32) -> i32"));
    assert!(content.contains("fn scale(x: f64, factor: f64) -> f64"));
    // The generated file must include the header via hicc::cpp! so that
    // namespace-qualified signatures compile with hicc-build.
    assert!(content.contains("hicc::cpp!"));
    assert!(content.contains("#include \"mylib.hpp\""));

    // LD_PRELOAD hook should capture header usage.
    let captured = tmp
        .path()
        .join(".cpp2rust/default/meta/captured_headers.list");
    assert!(captured.exists(), "captured_headers.list should exist");
    let captured_content = std::fs::read_to_string(captured).unwrap();
    assert!(
        captured_content.contains(h.to_str().unwrap()),
        "captured headers should contain input header path"
    );
}

#[test]
fn init_build_cmd_supports_shell_quoting() {
    let tmp = TempDir::new().unwrap();
    let header_path = write_header(&tmp, "quoted.hpp", "int quoted_add(int a, int b);");

    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "mylib",
            "--",
            "sh",
            "-c",
            "clang -x c++ -fsyntax-only quoted.hpp",
        ])
        .assert()
        .success();

    let ffi = tmp.path().join(".cpp2rust/default/rust/src/ffi_quoted.rs");
    assert!(ffi.exists(), "ffi_quoted.rs should exist");
    let ffi_content = std::fs::read_to_string(&ffi).unwrap();
    assert!(
        ffi_content.contains("fn quoted_add(a: i32, b: i32) -> i32"),
        "generated ffi should contain quoted_add binding"
    );

    let captured = tmp
        .path()
        .join(".cpp2rust/default/meta/captured_headers.list");
    assert!(captured.exists(), "captured_headers.list should exist");
    let captured_content = std::fs::read_to_string(captured).unwrap();
    assert!(
        captured_content.contains(header_path.to_str().unwrap()),
        "captured headers should contain header from quoted capture-cmd"
    );
}

#[test]
fn init_overloaded_functions_get_numeric_suffix() {
    let tmp = TempDir::new().unwrap();
    let h = write_header(
        &tmp,
        "over.hpp",
        r#"
        void process(int value);
        void process(double value);
        void process(const char* value);
        "#,
    );

    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "mylib",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            h.to_str().unwrap(),
        ])
        .assert()
        .success();

    let ffi = tmp
        .path()
        .join(".cpp2rust/default/rust/src/ffi_over.rs");
    let content = std::fs::read_to_string(&ffi).unwrap();

    // First overload keeps plain name.
    assert!(
        content.contains("fn process("),
        "first overload should keep 'process'"
    );
    // Second overload gets _2 suffix.
    assert!(
        content.contains("fn process_2("),
        "second overload should be 'process_2'"
    );
    // Third overload gets _3 suffix.
    assert!(
        content.contains("fn process_3("),
        "third overload should be 'process_3'"
    );
}

#[test]
fn init_namespace_qualified_signature() {
    let tmp = TempDir::new().unwrap();
    let h = write_header(
        &tmp,
        "ns.hpp",
        r#"
        namespace myns { int add(int a, int b); }
        "#,
    );

    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "myns",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            h.to_str().unwrap(),
        ])
        .assert()
        .success();

    let ffi = tmp
        .path()
        .join(".cpp2rust/default/rust/src/ffi_ns.rs");
    let content = std::fs::read_to_string(&ffi).unwrap();
    // The C++ signature in the attribute should be namespace-qualified.
    assert!(
        content.contains("myns::add"),
        "C++ signature should be namespace-qualified"
    );
}

#[test]
fn init_class_generates_import_class_and_import_lib() {
    let tmp = TempDir::new().unwrap();
    let h = write_header(
        &tmp,
        "widget.hpp",
        r#"
        class Widget {
        public:
            void update(double x, double y);
            int getId() const;
            static int instanceCount();
        };
        "#,
    );

    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "widget",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            h.to_str().unwrap(),
        ])
        .assert()
        .success();

    let ffi = tmp
        .path()
        .join(".cpp2rust/default/rust/src/ffi_widget.rs");
    let content = std::fs::read_to_string(&ffi).unwrap();

    // Instance methods go into import_class!
    assert!(content.contains("import_class!"), "should have import_class!");
    assert!(
        content.contains("class Widget {"),
        "should declare Widget class"
    );
    assert!(
        content.contains("fn update(&mut self"),
        "update should take &mut self"
    );
    assert!(
        content.contains("fn get_id(&self)"),
        "const getId should take &self"
    );

    // Static methods go into import_lib!
    assert!(content.contains("import_lib!"), "should have import_lib!");
    assert!(
        content.contains("class Widget;"),
        "should forward-declare Widget"
    );
    // Static method appears as a free function (not inside import_class!).
    assert!(
        content.contains("fn widget_instance_count()"),
        "static method should be a free fn in import_lib!"
    );
}

#[test]
fn init_creates_cargo_toml_with_hicc() {
    let tmp = TempDir::new().unwrap();
    let h = write_header(&tmp, "simple.hpp", "void foo();");

    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "mylib",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            h.to_str().unwrap(),
        ])
        .assert()
        .success();

    let cargo_toml = tmp.path().join(".cpp2rust/default/rust/Cargo.toml");
    let content = std::fs::read_to_string(cargo_toml).unwrap();
    assert!(content.contains("hicc"));
    assert!(content.contains("hicc-build"));
}

#[test]
fn init_creates_build_rs() {
    let tmp = TempDir::new().unwrap();
    let h = write_header(&tmp, "simple.hpp", "void foo();");

    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "mylib",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            h.to_str().unwrap(),
        ])
        .assert()
        .success();

    let build_rs = tmp.path().join(".cpp2rust/default/rust/build.rs");
    assert!(build_rs.exists());
    let content = std::fs::read_to_string(build_rs).unwrap();
    assert!(content.contains("hicc_build"));
}

#[test]
fn init_custom_feature() {
    let tmp = TempDir::new().unwrap();
    let h = write_header(&tmp, "simple.hpp", "void foo();");

    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--feature",
            "myfeature",
            "--link",
            "mylib",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            h.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(tmp
        .path()
        .join(".cpp2rust/myfeature/rust/src/ffi_simple.rs")
        .exists());
}

// ---------------------------------------------------------------------------
// merge command
// ---------------------------------------------------------------------------

#[test]
fn merge_without_init_fails() {
    let tmp = TempDir::new().unwrap();
    // Create the .cpp2rust dir but not the feature dir.
    std::fs::create_dir(tmp.path().join(".cpp2rust")).unwrap();

    bin()
        .current_dir(tmp.path())
        .args(["merge"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn merge_produces_merged_ffi() {
    let tmp = TempDir::new().unwrap();

    // Create two headers.
    let h1 = write_header(&tmp, "lib1.hpp", "int add(int a, int b);");
    let h2 = write_header(&tmp, "lib2.hpp", "void log(const char* msg);");

    // Init with both.
    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "mylib",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            h1.to_str().unwrap(),
            h2.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Merge.
    bin()
        .current_dir(tmp.path())
        .args(["merge"])
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ cpp2rust-demo merge completed"));

    let merged = tmp
        .path()
        .join(".cpp2rust/default/rust/src/merged_ffi.rs");
    assert!(merged.exists(), "merged_ffi.rs should exist");

    let content = std::fs::read_to_string(&merged).unwrap();
    // Should contain items from both headers.
    assert!(content.contains("fn add("));
    assert!(content.contains("fn log("));
    // Should have exactly one import_lib! block.
    assert_eq!(
        content.matches("import_lib!").count(),
        1,
        "should have exactly one import_lib! block"
    );
}

#[test]
fn merge_deduplicates_class_forward_decls() {
    let tmp = TempDir::new().unwrap();

    // Two headers that both reference the same class.
    let h1 = write_header(
        &tmp,
        "a.hpp",
        r#"class Widget {
        public:
            void update(double x, double y);
        };"#,
    );
    let h2 = write_header(&tmp, "b.hpp", "int add(int a, int b);");

    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "mylib",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            h1.to_str().unwrap(),
            h2.to_str().unwrap(),
        ])
        .assert()
        .success();

    bin()
        .current_dir(tmp.path())
        .args(["merge"])
        .assert()
        .success();

    let content = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/merged_ffi.rs"),
    )
    .unwrap();

    // "class Widget;" should appear exactly once in import_lib!
    let count = content.matches("class Widget;").count();
    assert_eq!(
        count, 1,
        "Widget forward decl should appear once, got {}",
        count
    );
}

#[test]
fn merge_updates_build_rs_to_merged_ffi() {
    let tmp = TempDir::new().unwrap();
    let h = write_header(&tmp, "simple.hpp", "void foo();");

    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "mylib",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            h.to_str().unwrap(),
        ])
        .assert()
        .success();

    bin()
        .current_dir(tmp.path())
        .args(["merge"])
        .assert()
        .success();

    let build_rs = std::fs::read_to_string(
        tmp.path().join(".cpp2rust/default/rust/build.rs"),
    )
    .unwrap();
    assert!(
        build_rs.contains("merged_ffi.rs"),
        "build.rs should reference merged_ffi.rs after merge"
    );
}

#[test]
fn merge_consolidates_cpp_includes() {
    let tmp = TempDir::new().unwrap();
    let h1 = write_header(&tmp, "lib1.hpp", "int add(int a, int b);");
    let h2 = write_header(&tmp, "lib2.hpp", "void log(const char* msg);");

    bin()
        .current_dir(tmp.path())
        .args([
            "init", "--link", "mylib", "--", "clang", "-x", "c++", "-fsyntax-only",
            h1.to_str().unwrap(),
            h2.to_str().unwrap(),
        ])
        .assert()
        .success();

    bin()
        .current_dir(tmp.path())
        .args(["merge"])
        .assert()
        .success();

    let merged = std::fs::read_to_string(
        tmp.path().join(".cpp2rust/default/rust/src/merged_ffi.rs"),
    )
    .unwrap();

    // Both headers should be included in a single hicc::cpp! block.
    assert!(merged.contains("hicc::cpp!"), "merged file should have hicc::cpp! block");
    assert!(merged.contains("#include \"lib1.hpp\""));
    assert!(merged.contains("#include \"lib2.hpp\""));
    // Should have exactly one hicc::cpp! block (consolidated).
    assert_eq!(merged.matches("hicc::cpp!").count(), 1,
        "should have exactly one consolidated hicc::cpp! block");
}

// ---------------------------------------------------------------------------
// cargo check integration tests (verify generated code is valid hicc input)
// ---------------------------------------------------------------------------

/// Type-mapping verification: class pointer and reference types (`const T&`,
/// `T&`, `const T*`, `T*`) used as function parameters and return values must
/// compile with hicc-build.
///
/// Covers both primitive-type (`int`) and user-defined class-type (`Point`)
/// scenarios using a global-scope class, so clang's qualType is already
/// unambiguous and no namespace qualification is required.
#[test]
fn cargo_check_class_reference_and_pointer_types() {
    if Command::new("cargo").arg("--version").output().is_err() {
        eprintln!("Skipping cargo-check test: cargo not found in PATH");
        return;
    }

    let tmp = TempDir::new().unwrap();

    // Header that exercises all four reference/pointer combinations for both
    // primitives and a class type.
    let header_content = r#"
#pragma once

class Point {
public:
    // Primitive reference / pointer parameters
    void set_x(const int& v);
    void add_to_x(int& v);
    void fill_array(int* buf, const int* src);

    // Class reference / pointer parameters
    void copy_from(const Point& other);
    void swap_with(Point& other);
    Point* clone() const;
    const Point* origin() const;
};

// Free functions with class parameters
void translate(Point& p, const Point& delta);
Point* create_point(int x, int y);
const Point* get_origin();
"#;
    let h = write_header(&tmp, "geometry.hpp", header_content);

    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "geometry",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            h.to_str().unwrap(),
        ])
        .assert()
        .success();

    bin()
        .current_dir(tmp.path())
        .args(["merge"])
        .assert()
        .success();

    let rust_proj = tmp.path().join(".cpp2rust/default/rust");
    let check_output = Command::new("cargo")
        .args(["check", "--message-format=short"])
        .current_dir(&rust_proj)
        .output()
        .expect("cargo should be available since we checked above");

    if !check_output.status.success() {
        eprintln!("=== cargo check stderr ===");
        eprintln!("{}", String::from_utf8_lossy(&check_output.stderr));
        eprintln!("=== generated merged_ffi.rs ===");
        let merged = rust_proj.join("src/merged_ffi.rs");
        if merged.exists() {
            eprintln!("{}", std::fs::read_to_string(merged).unwrap());
        }
        eprintln!("=== generated build.rs ===");
        let build_rs = rust_proj.join("build.rs");
        if build_rs.exists() {
            eprintln!("{}", std::fs::read_to_string(build_rs).unwrap());
        }
        panic!("cargo check failed for class/reference/pointer type mappings");
    }
}

/// Type-mapping verification: class types as return values and mixed
/// namespace + class scenarios must compile with hicc-build.
#[test]
fn cargo_check_class_return_values_and_namespace_classes() {
    if Command::new("cargo").arg("--version").output().is_err() {
        eprintln!("Skipping cargo-check test: cargo not found in PATH");
        return;
    }

    let tmp = TempDir::new().unwrap();

    let header_content = r#"
#pragma once

namespace geo {
    class Vec2 {
    public:
        double x() const;
        double y() const;
        void set(double x, double y);

        // Class return value (by pointer)
        Vec2* normalize() const;

        // Static factory
        static Vec2* zero();
    };

    // Free functions returning class pointers
    Vec2* lerp(const Vec2* a, const Vec2* b, double t);
    void scale(Vec2& v, double factor);
}
"#;
    let h = write_header(&tmp, "vec2.hpp", header_content);

    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "geo",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            h.to_str().unwrap(),
        ])
        .assert()
        .success();

    bin()
        .current_dir(tmp.path())
        .args(["merge"])
        .assert()
        .success();

    let rust_proj = tmp.path().join(".cpp2rust/default/rust");
    let check_output = Command::new("cargo")
        .args(["check", "--message-format=short"])
        .current_dir(&rust_proj)
        .output()
        .expect("cargo should be available since we checked above");

    if !check_output.status.success() {
        eprintln!("=== cargo check stderr ===");
        eprintln!("{}", String::from_utf8_lossy(&check_output.stderr));
        eprintln!("=== generated merged_ffi.rs ===");
        let merged = rust_proj.join("src/merged_ffi.rs");
        if merged.exists() {
            eprintln!("{}", std::fs::read_to_string(merged).unwrap());
        }
        eprintln!("=== generated build.rs ===");
        let build_rs = rust_proj.join("build.rs");
        if build_rs.exists() {
            eprintln!("{}", std::fs::read_to_string(build_rs).unwrap());
        }
        panic!("cargo check failed for namespace + class return-value type mappings");
    }
}
///
/// This proves the generated `hicc::cpp!` include + `hicc::import_lib!` macros
/// are accepted by hicc-build, not just that the text was produced correctly.
#[test]
fn generated_project_passes_cargo_check() {
    // Skip if cargo is not available (e.g. unusual CI environments).
    if Command::new("cargo").arg("--version").output().is_err() {
        eprintln!("Skipping cargo-check test: cargo not found in PATH");
        return;
    }

    let tmp = TempDir::new().unwrap();

    // A minimal header with a namespace-qualified free function.
    // Using a namespace is the hardest case: hicc needs the `hicc::cpp!`
    // include block to know about the namespace when compiling the adapter.
    let header_content = r#"
#pragma once
namespace mathlib {
    int add(int a, int b);
    double multiply(double x, double y);
}
"#;
    let h = write_header(&tmp, "mathlib.hpp", header_content);

    // Run init.
    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "mathlib",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            h.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Run merge.
    bin()
        .current_dir(tmp.path())
        .args(["merge"])
        .assert()
        .success();

    // Run `cargo check` on the generated project.
    let rust_proj = tmp.path().join(".cpp2rust/default/rust");
    let check_output = Command::new("cargo")
        .args(["check", "--message-format=short"])
        .current_dir(&rust_proj)
        .output()
        .expect("cargo should be available since we checked above");

    if !check_output.status.success() {
        eprintln!("=== cargo check stderr ===");
        eprintln!("{}", String::from_utf8_lossy(&check_output.stderr));
        eprintln!("=== generated merged_ffi.rs ===");
        let merged = rust_proj.join("src/merged_ffi.rs");
        if merged.exists() {
            eprintln!("{}", std::fs::read_to_string(merged).unwrap());
        }
        eprintln!("=== generated build.rs ===");
        let build_rs = rust_proj.join("build.rs");
        if build_rs.exists() {
            eprintln!("{}", std::fs::read_to_string(build_rs).unwrap());
        }
        panic!("cargo check failed on generated project");
    }
}
