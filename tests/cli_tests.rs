// Integration tests for the cpp2rust-demo CLI.
//
// These tests run the compiled binary against real C++ translation units
// (using the `clang` binary on the host) and verify the generated output.

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

// Helper: write a C++ translation unit that includes a header.
fn write_translation_unit(dir: &TempDir, name: &str, header: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, format!("#include \"{}\"\n", header)).unwrap();
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
fn init_nonexistent_input_file_fails() {
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
            "does_not_exist.cpp",
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
    write_header(
        &tmp,
        "mylib.hpp",
        r#"
        int add(int a, int b);
        double scale(double x, double factor);
        "#,
    );
    let tu = write_translation_unit(&tmp, "mylib.cpp", "mylib.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ cpp2rust-demo init completed"));

    // Check that grouped semantic files exist.
    let ffi = tmp
        .path()
        .join(".cpp2rust/default/rust/src/mod_mylib/free/fn_mylib.rs");
    assert!(ffi.exists(), "mod_mylib/free/fn_mylib.rs should exist");
    let include = tmp
        .path()
        .join(".cpp2rust/default/rust/src/mod_mylib/include/mod.rs");
    assert!(include.exists(), "mod_mylib/include/mod.rs should exist");
    assert!(tmp
        .path()
        .join(".cpp2rust/default/rust/src/mod_mylib/meta.json")
        .exists());

    let content = std::fs::read_to_string(&ffi).unwrap();
    assert!(content.contains("import_lib!"));
    assert!(content.contains("link_name = \"mylib\""));
    assert!(content.contains("fn add(a: i32, b: i32) -> i32"));
    assert!(content.contains("fn scale(x: f64, factor: f64) -> f64"));
    let include_content = std::fs::read_to_string(&include).unwrap();
    // The generated file must include the header via hicc::cpp! so that
    // namespace-qualified signatures compile with hicc-build.
    assert!(include_content.contains("hicc::cpp!"));
    assert!(include_content.contains("#include \"mylib.cpp.cpp2rust\""));

    // LD_PRELOAD hook should capture middleware file.
    let captured = tmp.path().join(".cpp2rust/default/cpp/mylib.cpp.cpp2rust");
    assert!(captured.exists(), "mylib.cpp.cpp2rust should exist");

    // Interactive middleware selection should produce selected_files.json.
    let selected = tmp
        .path()
        .join(".cpp2rust/default/meta/selected_files.json");
    assert!(selected.exists(), "selected_files.json should exist");
    let selected_content = std::fs::read_to_string(selected).unwrap();
    assert!(
        selected_content.contains("mylib.cpp.cpp2rust"),
        "selected_files.json should record chosen middleware files"
    );
}

#[test]
fn init_build_cmd_via_sh_c() {
    let tmp = TempDir::new().unwrap();
    write_header(&tmp, "quoted.hpp", "int quoted_add(int a, int b);");
    write_translation_unit(&tmp, "quoted.cpp", "quoted.hpp");

    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "mylib",
            "--",
            "sh",
            "-c",
            "clang -x c++ -fsyntax-only quoted.cpp",
        ])
        .assert()
        .success();

    let ffi = tmp
        .path()
        .join(".cpp2rust/default/rust/src/mod_quoted/free/fn_quoted.rs");
    assert!(ffi.exists(), "mod_quoted/free/fn_quoted.rs should exist");
    let ffi_content = std::fs::read_to_string(&ffi).unwrap();
    assert!(
        ffi_content.contains("fn quoted_add(a: i32, b: i32) -> i32"),
        "generated ffi should contain quoted_add binding"
    );

    let captured = tmp
        .path()
        .join(".cpp2rust/default/meta/selected_files.json");
    assert!(captured.exists(), "selected_files.json should exist");
    let captured_content = std::fs::read_to_string(captured).unwrap();
    assert!(
        captured_content.contains("quoted.cpp.cpp2rust"),
        "selected middleware should contain output from quoted capture-cmd"
    );
}

#[test]
fn init_overloaded_functions_get_numeric_suffix() {
    let tmp = TempDir::new().unwrap();
    write_header(
        &tmp,
        "over.hpp",
        r#"
        void process(int value);
        void process(double value);
        void process(const char* value);
        "#,
    );
    let tu = write_translation_unit(&tmp, "over.cpp", "over.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let ffi = tmp
        .path()
        .join(".cpp2rust/default/rust/src/mod_over/free/fn_over.rs");
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
    write_header(
        &tmp,
        "ns.hpp",
        r#"
        namespace myns { int add(int a, int b); }
        "#,
    );
    let tu = write_translation_unit(&tmp, "ns.cpp", "ns.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let ffi = tmp
        .path()
        .join(".cpp2rust/default/rust/src/mod_ns/free/fn_ns.rs");
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
    write_header(
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
    let tu = write_translation_unit(&tmp, "widget.cpp", "widget.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let class_ffi = tmp
        .path()
        .join(".cpp2rust/default/rust/src/mod_widget/class/cls_widget.rs");
    let method_ffi = tmp
        .path()
        .join(".cpp2rust/default/rust/src/mod_widget/method/mtd_widget.rs");
    let free_ffi = tmp
        .path()
        .join(".cpp2rust/default/rust/src/mod_widget/free/fn_widget.rs");
    let class_content = std::fs::read_to_string(&class_ffi).unwrap();
    let method_content = std::fs::read_to_string(&method_ffi).unwrap();
    let free_content = std::fs::read_to_string(&free_ffi).unwrap();

    // Class-level metadata stays in class/, method bindings go into method/.
    assert!(
        class_content.contains("CLASS_COUNT"),
        "class module should expose class-level metadata"
    );
    assert!(
        class_content.contains("CLASS_NAMES"),
        "class module should expose class name list"
    );
    assert!(
        class_content.contains("CLASS_METHODS"),
        "class module should expose class-method relation index"
    );
    assert!(
        class_content.contains("pub fn class_methods"),
        "class module should expose structured class-method accessors"
    );
    assert!(
        method_content.contains("import_class!"),
        "method module should have import_class!"
    );
    assert!(
        method_content.contains("class Widget {"),
        "should declare Widget class"
    );
    assert!(
        method_content.contains("fn update(&mut self"),
        "update should take &mut self"
    );
    assert!(
        method_content.contains("fn get_id(&self)"),
        "const getId should take &self"
    );

    // Static methods go into import_lib!
    assert!(
        free_content.contains("import_lib!"),
        "should have import_lib!"
    );
    assert!(
        free_content.contains("class Widget;"),
        "should forward-declare Widget"
    );
    // Static method appears as a free function (not inside import_class!).
    assert!(
        free_content.contains("fn widget_instance_count()"),
        "static method should be a free fn in import_lib!"
    );
}

#[test]
fn init_free_only_group_conditional_exports() {
    let tmp = TempDir::new().unwrap();
    write_header(&tmp, "free_only.hpp", "int ping(int v);");
    let tu = write_translation_unit(&tmp, "free_only.cpp", "free_only.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let group_mod = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_free_only/mod.rs"),
    )
    .unwrap();
    assert!(group_mod.contains("pub mod free;"));
    assert!(group_mod.contains("pub use free::*;"));
    assert!(!group_mod.contains("pub mod class;"));
    assert!(!group_mod.contains("pub mod method;"));

    let types_mod = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_free_only/types/mod.rs"),
    )
    .unwrap();
    assert!(types_mod.contains("CPP_TYPES"));
    assert!(types_mod.contains("CPP_RUST_TYPE_MAPPINGS"));
    assert!(types_mod.contains("pub fn rust_type_for"));

    let common_includes = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/common/includes.rs"),
    )
    .unwrap();
    assert!(common_includes.contains("MIDDLEWARE_FILES"));
    assert!(common_includes.contains("MIDDLEWARE_BASENAMES"));
    assert!(common_includes.contains("MIDDLEWARE_FILE_BASENAME_PAIRS"));
    assert!(common_includes.contains("INCLUDE_DIRS"));
    assert!(common_includes.contains("CPP_INCLUDE_LINES"));
    assert!(common_includes.contains("pub fn include_line_for"));

    let build_rs =
        std::fs::read_to_string(tmp.path().join(".cpp2rust/default/rust/build.rs")).unwrap();
    assert!(build_rs.contains("src/mod_free_only/free/fn_free_only.rs"));
    assert!(!build_rs.contains("src/mod_free_only/class/cls_free_only.rs"));
}

#[test]
fn init_class_only_group_conditional_exports() {
    let tmp = TempDir::new().unwrap();
    write_header(
        &tmp,
        "class_only.hpp",
        r#"
        class Counter {
        public:
            void inc();
            int value() const;
        };
        "#,
    );
    let tu = write_translation_unit(&tmp, "class_only.cpp", "class_only.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let group_mod = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_class_only/mod.rs"),
    )
    .unwrap();
    assert!(group_mod.contains("pub mod class;"));
    assert!(group_mod.contains("pub use class::*;"));
    assert!(group_mod.contains("pub mod method;"));
    assert!(group_mod.contains("pub use method::*;"));
    // class-only groups still keep free/import_lib for class forward declarations/static methods.
    assert!(group_mod.contains("pub mod free;"));

    let build_rs =
        std::fs::read_to_string(tmp.path().join(".cpp2rust/default/rust/build.rs")).unwrap();
    assert!(build_rs.contains("src/mod_class_only/class/cls_class_only.rs"));
    assert!(build_rs.contains("src/mod_class_only/method/mtd_class_only.rs"));
    assert!(build_rs.contains("src/mod_class_only/free/fn_class_only.rs"));
}

#[test]
fn init_no_declarations_group_generates_include_only_active_files() {
    let tmp = TempDir::new().unwrap();
    write_header(&tmp, "empty.hpp", "#pragma once\n");
    let tu = write_translation_unit(&tmp, "empty.cpp", "empty.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let group_mod = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_empty/mod.rs"),
    )
    .unwrap();
    assert!(group_mod.contains("pub mod include;"));
    assert!(group_mod.contains("pub mod types;"));
    assert!(!group_mod.contains("pub mod free;"));
    assert!(!group_mod.contains("pub mod class;"));
    assert!(!group_mod.contains("pub mod method;"));
    assert!(!tmp
        .path()
        .join(".cpp2rust/default/rust/src/mod_empty/global")
        .exists());

    let build_rs =
        std::fs::read_to_string(tmp.path().join(".cpp2rust/default/rust/build.rs")).unwrap();
    assert!(build_rs.contains("src/mod_empty/include/mod.rs"));
    assert!(!build_rs.contains("src/mod_empty/free/fn_empty.rs"));
    assert!(!build_rs.contains("src/mod_empty/class/cls_empty.rs"));
}

#[test]
fn init_no_link_skips_unsupported_members_and_reports_reasons() {
    let tmp = TempDir::new().unwrap();
    write_header(
        &tmp,
        "unsupported.hpp",
        r#"
        template <typename T>
        struct Box { T value; };

        class Api {
        public:
            Api();
            virtual ~Api();
            virtual int read() = 0;
            int operator[](int idx) const;
            static int stable();
        };

        Api operator+(const Api& lhs, const Api& rhs);
        int fill(int **out, const char **name);
        "#,
    );
    let tu = write_translation_unit(&tmp, "unsupported.cpp", "unsupported.hpp");

    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "rapidjson",
            "--no-link",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let build_rs =
        std::fs::read_to_string(tmp.path().join(".cpp2rust/default/rust/build.rs")).unwrap();
    assert!(!build_rs.contains("cargo::rustc-link-lib=rapidjson"));

    let free = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_unsupported/free/fn_unsupported.rs"),
    )
    .unwrap();
    assert!(free.contains("fn fill(out: *mut *mut i32, name: *mut *const i8) -> i32"));
    assert!(!free.contains("operator"));

    let report = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/meta/init-interface-report.md"),
    )
    .unwrap();
    assert!(report.contains("## Skipped declarations"));
    assert!(report.contains("template_decl"));
    assert!(report.contains("constructor"));
    assert!(report.contains("destructor"));
    assert!(
        report.contains("pure_virtual")
            || report.contains("virtual")
            || report.contains("unsupported_type")
    );
    assert!(report.contains("operator_overload"));
}

#[test]
fn init_skips_free_function_with_template_instance_type() {
    let tmp = TempDir::new().unwrap();
    write_header(
        &tmp,
        "templated.hpp",
        r#"
        template <typename T>
        struct Holder {
            T value;
        };

        int regular(int v);
        int use_holder(Holder<int>* h);
        "#,
    );
    let tu = write_translation_unit(&tmp, "templated.cpp", "templated.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let free = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_templated/free/fn_templated.rs"),
    )
    .unwrap();
    assert!(free.contains("fn regular(v: i32) -> i32"));
    assert!(!free.contains("use_holder"));

    let report = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/meta/init-interface-report.md"),
    )
    .unwrap();
    assert!(report.contains("unsupported_type"));
    assert!(report.contains("use_holder"));
}

#[test]
fn init_creates_cargo_toml_with_hicc() {
    let tmp = TempDir::new().unwrap();
    write_header(&tmp, "simple.hpp", "void foo();");
    let tu = write_translation_unit(&tmp, "simple.cpp", "simple.hpp");

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
            tu.to_str().unwrap(),
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
    write_header(&tmp, "simple.hpp", "void foo();");
    let tu = write_translation_unit(&tmp, "simple.cpp", "simple.hpp");

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
            tu.to_str().unwrap(),
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
    write_header(&tmp, "simple.hpp", "void foo();");
    let tu = write_translation_unit(&tmp, "simple.cpp", "simple.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(tmp
        .path()
        .join(".cpp2rust/myfeature/rust/src/mod_simple/free/fn_simple.rs")
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

    // Create two headers and matching translation units.
    write_header(&tmp, "lib1.hpp", "int add(int a, int b);");
    write_header(&tmp, "lib2.hpp", "void log(const char* msg);");
    let tu1 = write_translation_unit(&tmp, "lib1.cpp", "lib1.hpp");
    let tu2 = write_translation_unit(&tmp, "lib2.cpp", "lib2.hpp");
    let build_cmd = format!(
        "clang -x c++ -fsyntax-only {} && clang -x c++ -fsyntax-only {}",
        tu1.display(),
        tu2.display()
    );

    // Init with both.
    bin()
        .current_dir(tmp.path())
        .args(["init", "--link", "mylib", "--", "sh", "-c", &build_cmd])
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
        .join(".cpp2rust/default/rust/src.2/merged_ffi.rs");
    assert!(merged.exists(), "merged_ffi.rs should exist");
    let src = tmp.path().join(".cpp2rust/default/rust/src");
    assert!(
        std::fs::symlink_metadata(&src)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false),
        "rust/src should be a symlink after merge"
    );
    assert!(
        tmp.path().join(".cpp2rust/default/rust/src.1").exists(),
        "rust/src.1 should preserve init output"
    );
    assert!(
        tmp.path()
            .join(".cpp2rust/default/rust/src.2/mod_lib1.rs")
            .exists(),
        "merge should emit per-group module files into src.2"
    );

    let content = std::fs::read_to_string(&merged).unwrap();
    // Should contain items from both headers.
    assert!(content.contains("fn add("));
    assert!(content.contains("fn log("));
    assert!(content.contains("MIDDLEWARE_FILES"));
    assert!(content.contains("INCLUDE_DIRS"));
    assert!(content.contains("CPP_TYPES"));
    assert!(content.contains("CPP_RUST_TYPE_MAPPINGS"));
    assert!(content.contains("CPP_INCLUDE_LINES"));
    assert!(content.contains("pub fn rust_type_for"));
    assert!(content.contains("pub fn include_line_for"));
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
    write_header(
        &tmp,
        "a.hpp",
        r#"class Widget {
        public:
            void update(double x, double y);
        };"#,
    );
    write_header(&tmp, "b.hpp", "int add(int a, int b);");
    let tu1 = write_translation_unit(&tmp, "a.cpp", "a.hpp");
    let tu2 = write_translation_unit(&tmp, "b.cpp", "b.hpp");
    let build_cmd = format!(
        "clang -x c++ -fsyntax-only {} && clang -x c++ -fsyntax-only {}",
        tu1.display(),
        tu2.display()
    );

    bin()
        .current_dir(tmp.path())
        .args(["init", "--link", "mylib", "--", "sh", "-c", &build_cmd])
        .assert()
        .success();

    bin()
        .current_dir(tmp.path())
        .args(["merge"])
        .assert()
        .success();

    let content =
        std::fs::read_to_string(tmp.path().join(".cpp2rust/default/rust/src/merged_ffi.rs"))
            .unwrap();

    // "class Widget;" should appear exactly once in import_lib!
    let count = content.matches("class Widget;").count();
    assert_eq!(
        count, 1,
        "Widget forward decl should appear once, got {}",
        count
    );
    assert!(
        content.contains("CLASS_NAMES"),
        "merged output should carry class semantic metadata"
    );
    assert!(
        content.contains("CLASS_METHODS"),
        "merged output should carry class-method semantic relationships"
    );
    assert!(
        content.contains("pub fn class_method_count"),
        "merged output should carry class semantic access helpers"
    );
}

#[test]
fn merge_updates_build_rs_to_merged_ffi() {
    let tmp = TempDir::new().unwrap();
    write_header(&tmp, "simple.hpp", "void foo();");
    let tu = write_translation_unit(&tmp, "simple.cpp", "simple.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    bin()
        .current_dir(tmp.path())
        .args(["merge"])
        .assert()
        .success();

    let build_rs =
        std::fs::read_to_string(tmp.path().join(".cpp2rust/default/rust/build.rs")).unwrap();
    assert!(
        build_rs.contains("merged_ffi.rs"),
        "build.rs should reference merged_ffi.rs after merge"
    );
    let src2_lib =
        std::fs::read_to_string(tmp.path().join(".cpp2rust/default/rust/src.2/lib.rs")).unwrap();
    assert!(
        src2_lib.contains("pub mod mod_simple"),
        "src.2/lib.rs should expose merged group modules"
    );
}

#[test]
fn merge_preserves_no_link_build_rs() {
    let tmp = TempDir::new().unwrap();
    write_header(&tmp, "simple.hpp", "void foo();");
    let tu = write_translation_unit(&tmp, "simple.cpp", "simple.hpp");

    bin()
        .current_dir(tmp.path())
        .args([
            "init",
            "--link",
            "mylib",
            "--no-link",
            "--",
            "clang",
            "-x",
            "c++",
            "-fsyntax-only",
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    bin()
        .current_dir(tmp.path())
        .args(["merge"])
        .assert()
        .success();

    let build_rs =
        std::fs::read_to_string(tmp.path().join(".cpp2rust/default/rust/build.rs")).unwrap();
    assert!(!build_rs.contains("cargo::rustc-link-lib=mylib"));
    assert!(build_rs.contains("cargo::rustc-link-lib=cpp2rust_adapter"));
    assert!(build_rs.contains("merged_ffi.rs"));
}

#[test]
fn merge_consolidates_cpp_includes() {
    let tmp = TempDir::new().unwrap();
    write_header(&tmp, "lib1.hpp", "int add(int a, int b);");
    write_header(&tmp, "lib2.hpp", "void log(const char* msg);");
    let tu1 = write_translation_unit(&tmp, "lib1.cpp", "lib1.hpp");
    let tu2 = write_translation_unit(&tmp, "lib2.cpp", "lib2.hpp");
    let build_cmd = format!(
        "clang -x c++ -fsyntax-only {} && clang -x c++ -fsyntax-only {}",
        tu1.display(),
        tu2.display()
    );

    bin()
        .current_dir(tmp.path())
        .args(["init", "--link", "mylib", "--", "sh", "-c", &build_cmd])
        .assert()
        .success();

    bin()
        .current_dir(tmp.path())
        .args(["merge"])
        .assert()
        .success();

    let merged =
        std::fs::read_to_string(tmp.path().join(".cpp2rust/default/rust/src/merged_ffi.rs"))
            .unwrap();

    // Both headers should be included in a single hicc::cpp! block.
    assert!(
        merged.contains("hicc::cpp!"),
        "merged file should have hicc::cpp! block"
    );
    assert!(merged.contains("#include \"lib1.cpp.cpp2rust\""));
    assert!(merged.contains("#include \"lib2.cpp.cpp2rust\""));
    // Should have exactly one hicc::cpp! block (consolidated).
    assert_eq!(
        merged.matches("hicc::cpp!").count(),
        1,
        "should have exactly one consolidated hicc::cpp! block"
    );
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
    write_header(&tmp, "geometry.hpp", header_content);
    let tu = write_translation_unit(&tmp, "geometry.cpp", "geometry.hpp");

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
            tu.to_str().unwrap(),
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
    write_header(&tmp, "vec2.hpp", header_content);
    let tu = write_translation_unit(&tmp, "vec2.cpp", "vec2.hpp");

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
            tu.to_str().unwrap(),
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
    write_header(&tmp, "mathlib.hpp", header_content);
    let tu = write_translation_unit(&tmp, "mathlib.cpp", "mathlib.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Run merge.
    bin()
        .current_dir(tmp.path())
        .args(["merge"])
        .assert()
        .success();

    let rust_proj = tmp.path().join(".cpp2rust/default/rust");
    let src = rust_proj.join("src");
    assert!(
        std::fs::symlink_metadata(&src)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false),
        "merge should keep rust/src as active symlink view"
    );
    let build_rs = std::fs::read_to_string(rust_proj.join("build.rs")).unwrap();
    assert!(
        build_rs.contains("src/merged_ffi.rs"),
        "build.rs should target active src/ view"
    );
    assert!(
        rust_proj.join("src/merged_ffi.rs").exists(),
        "active src/ view should resolve merged_ffi.rs after merge"
    );

    // Run `cargo check` on the generated project.
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

// ---------------------------------------------------------------------------
// Virtual method and abstract class extraction tests
// ---------------------------------------------------------------------------

/// Non-pure virtual methods are now extracted as regular hicc methods.
/// The class should still use `#[cpp(class = "...")]` (not `#[interface]`).
#[test]
fn init_virtual_methods_extracted() {
    let tmp = TempDir::new().unwrap();
    write_header(
        &tmp,
        "virtual.hpp",
        r#"
        class Engine {
        public:
            virtual int tick(int delta);
            virtual void reset();
            void status() const;
        };
        "#,
    );
    let tu = write_translation_unit(&tmp, "virtual.cpp", "virtual.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let method_src = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_virtual/method/mtd_virtual.rs"),
    )
    .unwrap();
    // All three public methods (including the two virtual ones) should be extracted.
    assert!(method_src.contains("fn tick(&mut self, delta: i32) -> i32"));
    assert!(method_src.contains("fn reset(&mut self)"));
    assert!(method_src.contains("fn status(&self)"));
    // The class is non-abstract → uses #[cpp(class = "Engine")]
    assert!(method_src.contains(r#"cpp(class = "Engine")"#));
    assert!(!method_src.contains("#[interface]"));

    let report = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/meta/init-interface-report.md"),
    )
    .unwrap();
    // Virtual methods should NOT be in the skipped-declarations table.
    assert!(
        !report.contains("| `virtual` |"),
        "non-pure virtual methods should no longer be listed as skipped"
    );
}

/// A fully-abstract C++ class (all public methods are pure-virtual) should be
/// emitted as a hicc `#[interface]` trait, not a concrete `#[cpp(class = "...")]`.
#[test]
fn init_abstract_class_generates_interface() {
    let tmp = TempDir::new().unwrap();
    write_header(
        &tmp,
        "abstract.hpp",
        r#"
        class IProcessor {
        public:
            virtual int process(int x) const = 0;
            virtual void reset() = 0;
        };
        "#,
    );
    let tu = write_translation_unit(&tmp, "abstract.cpp", "abstract.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let method_src = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_abstract/method/mtd_abstract.rs"),
    )
    .unwrap();
    // Fully-abstract class → #[interface] annotation.
    assert!(
        method_src.contains("#[interface]"),
        "fully-abstract class should use #[interface]"
    );
    assert!(
        !method_src.contains(r#"cpp(class = "IProcessor")"#),
        "abstract class should not use #[cpp(class = ...)]"
    );
    // Both pure-virtual methods should be included.
    assert!(method_src.contains("fn process(&self, x: i32) -> i32"));
    assert!(method_src.contains("fn reset(&mut self)"));

    // Abstract class should NOT appear as a forward decl in import_lib!
    let free_src = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_abstract/free/fn_abstract.rs"),
    )
    .unwrap_or_default();
    assert!(
        !free_src.contains("class IProcessor;"),
        "abstract class should not be forward-declared in import_lib!"
    );
}

/// Operator overload shim hints should appear in the interface report when
/// operator overloads are skipped.
#[test]
fn init_operator_overload_report_includes_shim_hints() {
    let tmp = TempDir::new().unwrap();
    write_header(
        &tmp,
        "ops.hpp",
        r#"
        class Vec {
        public:
            Vec operator+(const Vec& rhs) const;
            Vec& operator=(const Vec& rhs);
            int operator[](int idx) const;
            double get(int idx) const;
        };
        "#,
    );
    let tu = write_translation_unit(&tmp, "ops.cpp", "ops.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let report = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/meta/init-interface-report.md"),
    )
    .unwrap();
    // Skipped operators should trigger the shim hints section.
    assert!(report.contains("## Operator Overload Shim Hints"));
    assert!(report.contains("hicc::cpp!"));
    // Regular non-operator method should be extracted and not skipped.
    let method_src = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_ops/method/mtd_ops.rs"),
    )
    .unwrap();
    assert!(method_src.contains("fn get(&self, idx: i32) -> f64"));
}

// ---------------------------------------------------------------------------
// hicc capability expansion: constructor extraction
// ---------------------------------------------------------------------------

/// When a class has a public constructor with supported parameter types,
/// the generated `import_class!` block should include `ctor = "..."`.
#[test]
fn init_class_with_ctor_generates_ctor_attribute() {
    let tmp = TempDir::new().unwrap();
    write_header(
        &tmp,
        "widget_ctor.hpp",
        r#"
        class Widget {
        public:
            Widget(int id, double scale);
            void update(double x, double y);
            int getId() const;
        };
        "#,
    );
    let tu = write_translation_unit(&tmp, "widget_ctor.cpp", "widget_ctor.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let method_src = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_widget_ctor/method/mtd_widget_ctor.rs"),
    )
    .unwrap();

    // The import_class! block should include ctor = "...".
    assert!(
        method_src.contains("ctor = \"Widget("),
        "import_class! should include ctor attribute: {}",
        method_src
    );
    // Regular methods should still be present.
    assert!(method_src.contains("fn update(&mut self"));
    assert!(method_src.contains("fn get_id(&self)"));

    // Report should show the extracted constructor.
    let report = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/meta/init-interface-report.md"),
    )
    .unwrap();
    assert!(
        report.contains("### Constructors") || report.contains("Constructors"),
        "report should document extracted constructors"
    );
    assert!(
        report.contains("primary"),
        "report should label primary constructor"
    );
}

/// When a class has multiple constructors, the primary (fewest params) goes
/// into `import_class!`; additional ones become named factory functions in
/// `import_lib!`.
#[test]
fn init_class_multiple_ctors_generates_factory_functions() {
    let tmp = TempDir::new().unwrap();
    write_header(
        &tmp,
        "multi_ctor.hpp",
        r#"
        class Counter {
        public:
            Counter();
            Counter(int start);
            Counter(int start, int step);
            void inc();
            int value() const;
        };
        "#,
    );
    let tu = write_translation_unit(&tmp, "multi_ctor.cpp", "multi_ctor.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let method_src = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_multi_ctor/method/mtd_multi_ctor.rs"),
    )
    .unwrap();
    // Primary ctor (fewest params = 0-param default ctor) in import_class!.
    assert!(
        method_src.contains("ctor = \"Counter()\""),
        "primary ctor (0-param) should be in import_class!: {}",
        method_src
    );

    let free_src = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_multi_ctor/free/fn_multi_ctor.rs"),
    )
    .unwrap();
    // Extra ctors should become factory functions in import_lib!.
    assert!(
        free_src.contains("new_2") || free_src.contains("new_3"),
        "extra constructors should appear as factory fns in import_lib!: {}",
        free_src
    );
    assert!(
        free_src.contains("#[member(class"),
        "extra ctor factory should use #[member(...)] attribute: {}",
        free_src
    );
}

/// When constructors are declared in reverse param-count order (most params
/// first), the 0-param constructor should still be selected as the primary
/// `ctor = "..."` and the higher-param constructors should become factories.
#[test]
fn init_class_ctors_sorted_by_param_count() {
    let tmp = TempDir::new().unwrap();
    write_header(
        &tmp,
        "rev_ctor.hpp",
        r#"
        class Stack {
        public:
            Stack(int capacity, int flags);
            Stack(int capacity);
            Stack();
            void push(int val);
        };
        "#,
    );
    let tu = write_translation_unit(&tmp, "rev_ctor.cpp", "rev_ctor.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let method_src = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_rev_ctor/method/mtd_rev_ctor.rs"),
    )
    .unwrap();
    // Even though Stack() is declared last, it should be the primary ctor.
    assert!(
        method_src.contains("ctor = \"Stack()\""),
        "0-param ctor should be primary even when declared last: {}",
        method_src
    );

    let free_src = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_rev_ctor/free/fn_rev_ctor.rs"),
    )
    .unwrap();
    // The 1-param and 2-param ctors should become factory fns, NOT the 0-param.
    assert!(
        free_src.contains("new_2"),
        "non-primary ctors should be factory fns: {}",
        free_src
    );
    // The 0-param ctor (primary) should NOT appear as a factory fn.
    assert!(
        !free_src.contains("Stack()"),
        "primary ctor Stack() should not be repeated as a factory fn: {}",
        free_src
    );
}


/// When a class publicly inherits from another, the generated `import_class!`
/// block should use `class Foo: Base { ... }` syntax.
#[test]
fn init_class_inheritance_generates_bases_syntax() {
    let tmp = TempDir::new().unwrap();
    write_header(
        &tmp,
        "inherit.hpp",
        r#"
        class Shape {
        public:
            virtual double area() const = 0;
        };

        class Circle: public Shape {
        public:
            explicit Circle(double radius);
            double area() const;
            double radius() const;
        };
        "#,
    );
    let tu = write_translation_unit(&tmp, "inherit.cpp", "inherit.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let method_src = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_inherit/method/mtd_inherit.rs"),
    )
    .unwrap();

    // Shape should be abstract (#[interface]).
    assert!(
        method_src.contains("#[interface]"),
        "Shape (fully abstract) should use #[interface]: {}",
        method_src
    );
    // Circle should inherit from Shape in the hicc syntax.
    assert!(
        method_src.contains(": Shape"),
        "Circle should include base class in import_class!: {}",
        method_src
    );
    // Circle should have its own methods.
    assert!(method_src.contains("fn area(&self)"));
    assert!(method_src.contains("fn radius(&self)"));
}

// ---------------------------------------------------------------------------
// hicc capability expansion: @make_proxy for abstract classes
// ---------------------------------------------------------------------------

/// Fully-abstract classes should generate a `@make_proxy` binding in
/// `import_lib!` so Rust code can implement the interface.
#[test]
fn init_abstract_class_generates_make_proxy() {
    let tmp = TempDir::new().unwrap();
    write_header(
        &tmp,
        "listener.hpp",
        r#"
        class Listener {
        public:
            virtual void onEvent(int event_id) = 0;
            virtual bool shouldStop() const = 0;
        };
        "#,
    );
    let tu = write_translation_unit(&tmp, "listener.cpp", "listener.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let free_src = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_listener/free/fn_listener.rs"),
    )
    .unwrap();

    // @make_proxy binding should be generated.
    assert!(
        free_src.contains("@make_proxy"),
        "abstract class should have @make_proxy binding: {}",
        free_src
    );
    assert!(
        free_src.contains("#[interface(name = \"Listener\")]"),
        "make_proxy binding should have #[interface(name = ...)] attribute: {}",
        free_src
    );
    assert!(
        free_src.contains("new_listener_proxy"),
        "make_proxy should be named new_<snake>_proxy: {}",
        free_src
    );
    assert!(
        free_src.contains("hicc::Interface<Listener>"),
        "make_proxy fn should take hicc::Interface<T>: {}",
        free_src
    );
    // The free module should emit an actual hicc::cpp! block with the memory
    // header so @make_proxy compiles without manual edits.
    assert!(
        free_src.contains("hicc/std/memory.hpp"),
        "free module should include hicc/std/memory.hpp for @make_proxy: {}",
        free_src
    );
    assert!(
        free_src.contains("hicc::cpp!"),
        "free module should have a hicc::cpp! block (not just a comment) when abstract classes present: {}",
        free_src
    );

    // The @make_proxy comment hint should be in the report.
    let report = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/meta/init-interface-report.md"),
    )
    .unwrap();
    assert!(
        report.contains("@make_proxy") || report.contains("make_proxy"),
        "report should mention @make_proxy for abstract class: {}",
        report
    );
}

// ---------------------------------------------------------------------------
// hicc capability expansion: global variable extraction
// ---------------------------------------------------------------------------

/// Global variables at namespace scope should be extracted and generate
/// `#[cpp(data = "...")]` bindings in `import_lib!`.
#[test]
fn init_global_variable_generates_data_binding() {
    let tmp = TempDir::new().unwrap();
    write_header(
        &tmp,
        "globals.hpp",
        r#"
        extern int g_count;
        extern const double g_pi;
        extern const char* g_name;
        "#,
    );
    let tu = write_translation_unit(&tmp, "globals.cpp", "globals.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let free_src = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_globals/free/fn_globals.rs"),
    )
    .unwrap();

    // Mutable global: `&'static mut T`
    assert!(
        free_src.contains("#[cpp(data = \"g_count\")]"),
        "g_count should have #[cpp(data = ...)] binding: {}",
        free_src
    );
    assert!(
        free_src.contains("&'static mut i32"),
        "mutable g_count should return &'static mut i32: {}",
        free_src
    );

    // Const global: `&'static T`
    assert!(
        free_src.contains("#[cpp(data = \"g_pi\")]"),
        "g_pi should have #[cpp(data = ...)] binding: {}",
        free_src
    );
    assert!(
        free_src.contains("&'static f64"),
        "const g_pi should return &'static f64: {}",
        free_src
    );

    // Report should show global variables section.
    let report = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/meta/init-interface-report.md"),
    )
    .unwrap();
    assert!(
        report.contains("## Global Variables"),
        "report should have Global Variables section: {}",
        report
    );
    assert!(report.contains("g_count"));
    assert!(report.contains("g_pi"));
    // Report should include the "Rust fn name" column header.
    assert!(
        report.contains("Rust fn name"),
        "report Global Variables table should have Rust fn name column: {}",
        report
    );
}

/// Global variables inside namespaces should have their qualified name used in
/// `#[cpp(data = "ns::var")]`.
#[test]
fn init_namespaced_global_variable_uses_qualified_name() {
    let tmp = TempDir::new().unwrap();
    write_header(
        &tmp,
        "ns_globals.hpp",
        r#"
        namespace config {
            extern int max_retries;
            extern const double timeout_secs;
        }
        "#,
    );
    let tu = write_translation_unit(&tmp, "ns_globals.cpp", "ns_globals.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let free_src = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_ns_globals/free/fn_ns_globals.rs"),
    )
    .unwrap();

    // Qualified name should be used in the data attribute.
    assert!(
        free_src.contains("#[cpp(data = \"config::max_retries\")]"),
        "namespaced global should use qualified name: {}",
        free_src
    );
    assert!(
        free_src.contains("#[cpp(data = \"config::timeout_secs\")]"),
        "namespaced const global should use qualified name: {}",
        free_src
    );

    // meta.json should list the extracted globals.
    let meta_src = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_ns_globals/meta.json"),
    )
    .unwrap();
    assert!(
        meta_src.contains("config::max_retries"),
        "meta.json globals should include qualified global name: {}",
        meta_src
    );
    assert!(
        meta_src.contains("config::timeout_secs"),
        "meta.json globals should include qualified global name: {}",
        meta_src
    );
}

/// Global variables with camelCase C++ names should have their Rust accessor
/// function name converted to snake_case, matching how free functions work.
#[test]
fn init_camel_case_global_variable_uses_snake_case_rust_name() {
    let tmp = TempDir::new().unwrap();
    write_header(
        &tmp,
        "camel_globals.hpp",
        r#"
        extern int maxRetryCount;
        extern const double defaultTimeoutSecs;
        "#,
    );
    let tu = write_translation_unit(&tmp, "camel_globals.cpp", "camel_globals.hpp");

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
            tu.to_str().unwrap(),
        ])
        .assert()
        .success();

    let free_src = std::fs::read_to_string(
        tmp.path()
            .join(".cpp2rust/default/rust/src/mod_camel_globals/free/fn_camel_globals.rs"),
    )
    .unwrap();

    // Rust accessor functions must use snake_case names.
    assert!(
        free_src.contains("fn max_retry_count()"),
        "camelCase global should produce snake_case Rust fn name: {}",
        free_src
    );
    assert!(
        free_src.contains("fn default_timeout_secs()"),
        "camelCase const global should produce snake_case Rust fn name: {}",
        free_src
    );

    // C++ names in #[cpp(data)] should be the original C++ identifiers.
    assert!(
        free_src.contains("#[cpp(data = \"maxRetryCount\")]"),
        "#[cpp(data)] should use original C++ name: {}",
        free_src
    );
    assert!(
        free_src.contains("#[cpp(data = \"defaultTimeoutSecs\")]"),
        "#[cpp(data)] should use original C++ name: {}",
        free_src
    );
}
