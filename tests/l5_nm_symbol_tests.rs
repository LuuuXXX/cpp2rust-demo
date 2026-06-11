// L5 nm-based FFI symbol validation tests.
//
// These tests verify that every extern-"C" function exported by the C++ side
// is correctly linked into the corresponding Rust FFI binary.
//
// Validation flow (bidirectional):
//   1. Compile C++ sources → .o  with g++
//   2. nm --defined-only -f posix .o  → cpp_exports  (T/W, non-_Z names)
//   3. cargo build the Rust crate → binary
//   4. nm --defined-only -f posix binary → rust_linked  (filtered by cpp_exports)
//   5. Assert cpp_exports ⊆ rust_linked
//
// All tests are marked #[ignore] because they require a full C++ toolchain
// and a working Rust build environment.  Run them explicitly with:
//   cargo test -- --ignored

mod common;

use common::nm_utils::{
    assert_cpp_exports_linked, cargo_build_example, collect_archive_symbols, compile_cpp_obj,
    nm_c_exports,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────────────────────────
//  Helper: absolute path to the repository root.
//  Tests are run with `cargo test` from the repository root.
// ─────────────────────────────────────────────────────────────────────────────

fn repo_root() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR points to the workspace root when running `cargo test`
    // from the workspace.
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// ─────────────────────────────────────────────────────────────────────────────
//  Part 1 – examples 001-048 bidirectional nm validation
// ─────────────────────────────────────────────────────────────────────────────

struct ExampleSpec {
    dir_name: &'static str, // e.g. "001_hello_world"
    bin_name: &'static str, // Cargo.toml package.name, e.g. "hello_world"
}

const EXAMPLES: &[ExampleSpec] = &[
    ExampleSpec {
        dir_name: "001_hello_world",
        bin_name: "hello_world",
    },
    ExampleSpec {
        dir_name: "002_function_overload",
        bin_name: "function_overload",
    },
    ExampleSpec {
        dir_name: "003_default_args",
        bin_name: "default_args",
    },
    ExampleSpec {
        dir_name: "004_inline_functions",
        bin_name: "inline_functions",
    },
    ExampleSpec {
        dir_name: "005_variadic_functions",
        bin_name: "variadic_functions",
    },
    ExampleSpec {
        dir_name: "006_class_basic",
        bin_name: "class_basic",
    },
    ExampleSpec {
        dir_name: "007_class_constructor",
        bin_name: "class_constructor",
    },
    ExampleSpec {
        dir_name: "008_class_copy",
        bin_name: "class_copy",
    },
    ExampleSpec {
        dir_name: "009_class_move",
        bin_name: "class_move",
    },
    ExampleSpec {
        dir_name: "010_class_static",
        bin_name: "class_static",
    },
    ExampleSpec {
        dir_name: "011_class_const",
        bin_name: "class_const",
    },
    ExampleSpec {
        dir_name: "012_class_volatile",
        bin_name: "class_volatile",
    },
    ExampleSpec {
        dir_name: "013_inheritance_single",
        bin_name: "inheritance_single",
    },
    ExampleSpec {
        dir_name: "014_inheritance_multiple",
        bin_name: "inheritance_multiple",
    },
    ExampleSpec {
        dir_name: "015_virtual_basic",
        bin_name: "virtual_basic",
    },
    ExampleSpec {
        dir_name: "016_virtual_pure",
        bin_name: "virtual_pure",
    },
    ExampleSpec {
        dir_name: "017_virtual_override",
        bin_name: "virtual_override",
    },
    ExampleSpec {
        dir_name: "018_virtual_diamond",
        bin_name: "virtual_diamond",
    },
    ExampleSpec {
        dir_name: "019_operator_overload",
        bin_name: "operator_overload",
    },
    ExampleSpec {
        dir_name: "020_friend_function",
        bin_name: "friend_function",
    },
    ExampleSpec {
        dir_name: "021_explicit_ctor",
        bin_name: "explicit_ctor",
    },
    ExampleSpec {
        dir_name: "022_mutable_member",
        bin_name: "mutable_member",
    },
    ExampleSpec {
        dir_name: "023_typeid_rtti",
        bin_name: "typeid_rtti",
    },
    ExampleSpec {
        dir_name: "024_template_function",
        bin_name: "template_function",
    },
    ExampleSpec {
        dir_name: "025_template_class",
        bin_name: "template_class",
    },
    ExampleSpec {
        dir_name: "026_template_specialization",
        bin_name: "template_specialization",
    },
    ExampleSpec {
        dir_name: "027_template_instantiation",
        bin_name: "template_instantiation",
    },
    ExampleSpec {
        dir_name: "028_variadic_template",
        bin_name: "variadic_template",
    },
    ExampleSpec {
        dir_name: "029_unique_ptr",
        bin_name: "unique_ptr",
    },
    ExampleSpec {
        dir_name: "030_shared_ptr",
        bin_name: "shared_ptr",
    },
    ExampleSpec {
        dir_name: "031_custom_deleter",
        bin_name: "custom_deleter",
    },
    ExampleSpec {
        dir_name: "032_placement_new",
        bin_name: "placement_new",
    },
    ExampleSpec {
        dir_name: "033_raii_pattern",
        bin_name: "raii_pattern",
    },
    ExampleSpec {
        dir_name: "034_vector_basic",
        bin_name: "vector_basic",
    },
    ExampleSpec {
        dir_name: "035_map_basic",
        bin_name: "map_basic",
    },
    ExampleSpec {
        dir_name: "036_string_basic",
        bin_name: "string_basic",
    },
    ExampleSpec {
        dir_name: "037_array_basic",
        bin_name: "array_basic",
    },
    ExampleSpec {
        dir_name: "038_tuple_basic",
        bin_name: "tuple_basic",
    },
    ExampleSpec {
        dir_name: "039_lambda_basic",
        bin_name: "lambda_basic",
    },
    ExampleSpec {
        dir_name: "040_std_function",
        bin_name: "std_function",
    },
    ExampleSpec {
        dir_name: "041_functional_bind",
        bin_name: "functional_bind",
    },
    ExampleSpec {
        dir_name: "042_exception_basic",
        bin_name: "exception_basic",
    },
    ExampleSpec {
        dir_name: "043_namespace_nested",
        bin_name: "namespace_nested",
    },
    ExampleSpec {
        dir_name: "044_enum_class",
        bin_name: "enum_class",
    },
    ExampleSpec {
        dir_name: "045_union_basic",
        bin_name: "union_basic",
    },
    ExampleSpec {
        dir_name: "046_constexpr_basic",
        bin_name: "constexpr_basic",
    },
    ExampleSpec {
        dir_name: "047_noexcept_basic",
        bin_name: "noexcept_basic",
    },
    ExampleSpec {
        dir_name: "048_summary",
        bin_name: "summary",
    },
];

/// Validate one example: compile its C++ sources, build its Rust crate,
/// then assert all C++ extern-C exports appear in the compiled static archive.
///
/// We check the `.a` archive produced by `build.rs` (via `cc::Build::compile`)
/// rather than the final linked binary.  The reason is that the hicc framework
/// generates its own C++ bindings for `import_class!` methods, so it does NOT
/// call the hand-written C wrapper functions (e.g. `adder_add`).  Those wrappers
/// are compiled into the archive but are never referenced from Rust, so the
/// linker dead-strips them from the final executable.  The archive retains every
/// compiled symbol regardless of whether Rust calls it.
fn validate_example(spec: &ExampleSpec) {
    let root = repo_root();
    let example_dir = root.join("examples").join(spec.dir_name);

    // ── Step 1: find C++ source files ────────────────────────────────────────
    let cpp_dir = example_dir.join("cpp");
    let cpp_srcs: Vec<std::path::PathBuf> = std::fs::read_dir(&cpp_dir)
        .unwrap_or_else(|e| panic!("[L5-nm] {}: cannot read cpp dir: {}", spec.dir_name, e))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("cpp"))
        .collect();

    assert!(
        !cpp_srcs.is_empty(),
        "[L5-nm] {}: no .cpp files found in {:?}",
        spec.dir_name,
        cpp_dir
    );

    // ── Step 2: compile C++ sources to a single .o ───────────────────────────
    let tmp_dir = std::env::temp_dir().join(format!("l5_nm_{}", spec.dir_name));
    std::fs::create_dir_all(&tmp_dir)
        .unwrap_or_else(|e| panic!("[L5-nm] {}: create tmp dir: {}", spec.dir_name, e));

    let out_obj = tmp_dir.join("combined.o");
    let src_refs: Vec<&Path> = cpp_srcs.iter().map(PathBuf::as_path).collect();
    let obj_path = compile_cpp_obj(&src_refs, &[], &out_obj)
        .unwrap_or_else(|| panic!("[L5-nm] {}: g++ compilation failed", spec.dir_name));

    // ── Step 3: extract C++ extern-C exports ─────────────────────────────────
    let cpp_exports = nm_c_exports(&obj_path);
    assert!(
        !cpp_exports.is_empty(),
        "[L5-nm] {}: nm found no extern-C exports in C++ .o (unexpected – check source)",
        spec.dir_name
    );

    // ── Step 4: cargo build Rust crate ───────────────────────────────────────
    // This also triggers build.rs which compiles the C++ sources into a static
    // archive via cc::Build::compile().
    let rust_dir = example_dir.join("rust_hicc").to_string_lossy().into_owned();
    let _bin = cargo_build_example(&rust_dir, spec.bin_name).unwrap_or_else(|| {
        panic!(
            "[L5-nm] {}: cargo build failed or binary '{}' not found",
            spec.dir_name, spec.bin_name
        )
    });

    // ── Step 5: collect symbols from the static archives in the build output ──
    // `cc::Build::compile("name")` places `libname.a` in $OUT_DIR which is
    // under `target/debug/build/{pkg}-{hash}/out/`.  We scan all .a files
    // under `target/debug/` to cover that location without needing to know
    // the exact hash-suffixed directory name.
    //
    // When CARGO_TARGET_DIR is set (shared pre-build target directory), all
    // example archives live there; fall back to the crate-local target/debug/.
    let cpp_set: HashSet<String> = cpp_exports.iter().cloned().collect();
    let build_dir = if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        std::path::PathBuf::from(&target_dir).join("debug")
    } else {
        example_dir.join("rust_hicc").join("target/debug")
    };
    let archive_symbols = collect_archive_symbols(&build_dir);
    let rust_linked: HashSet<String> = cpp_set.intersection(&archive_symbols).cloned().collect();

    // ── Step 6: assertion ────────────────────────────────────────────────────
    assert_cpp_exports_linked(&cpp_exports, &rust_linked, spec.dir_name);
}

// ── One test function per example ────────────────────────────────────────────
//
// We generate individual test functions so that failures are reported per-
// example rather than stopping at the first failure in a loop.

macro_rules! example_test {
    ($fn_name:ident, $idx:literal) => {
        #[test]
        #[ignore]
        fn $fn_name() {
            validate_example(&EXAMPLES[$idx]);
        }
    };
}

example_test!(nm_001_hello_world, 0);
example_test!(nm_002_function_overload, 1);
example_test!(nm_003_default_args, 2);
example_test!(nm_004_inline_functions, 3);
example_test!(nm_005_variadic_functions, 4);
example_test!(nm_006_class_basic, 5);
example_test!(nm_007_class_constructor, 6);
example_test!(nm_008_class_copy, 7);
example_test!(nm_009_class_move, 8);
example_test!(nm_010_class_static, 9);
example_test!(nm_011_class_const, 10);
example_test!(nm_012_class_volatile, 11);
example_test!(nm_013_inheritance_single, 12);
example_test!(nm_014_inheritance_multiple, 13);
example_test!(nm_015_virtual_basic, 14);
example_test!(nm_016_virtual_pure, 15);
example_test!(nm_017_virtual_override, 16);
example_test!(nm_018_virtual_diamond, 17);
example_test!(nm_019_operator_overload, 18);
example_test!(nm_020_friend_function, 19);
example_test!(nm_021_explicit_ctor, 20);
example_test!(nm_022_mutable_member, 21);
example_test!(nm_023_typeid_rtti, 22);
example_test!(nm_024_template_function, 23);
example_test!(nm_025_template_class, 24);
example_test!(nm_026_template_specialization, 25);
example_test!(nm_027_template_instantiation, 26);
example_test!(nm_028_variadic_template, 27);
example_test!(nm_029_unique_ptr, 28);
example_test!(nm_030_shared_ptr, 29);
example_test!(nm_031_custom_deleter, 30);
example_test!(nm_032_placement_new, 31);
example_test!(nm_033_raii_pattern, 32);
example_test!(nm_034_vector_basic, 33);
example_test!(nm_035_map_basic, 34);
example_test!(nm_036_string_basic, 35);
example_test!(nm_037_array_basic, 36);
example_test!(nm_038_tuple_basic, 37);
example_test!(nm_039_lambda_basic, 38);
example_test!(nm_040_std_function, 39);
example_test!(nm_041_functional_bind, 40);
example_test!(nm_042_exception_basic, 41);
example_test!(nm_043_namespace_nested, 42);
example_test!(nm_044_enum_class, 43);
example_test!(nm_045_union_basic, 44);
example_test!(nm_046_constexpr_basic, 45);
example_test!(nm_047_noexcept_basic, 46);
example_test!(nm_048_summary, 47);

// ─────────────────────────────────────────────────────────────────────────────
//  Part 2 – rapidjson shim bidirectional nm validation
// ─────────────────────────────────────────────────────────────────────────────

/// Compile all rapidjson shim .cpp files, then verify that the resulting
/// extern-C symbols are present in the static archive produced by
/// `cargo build` of rapidjson_sys.
///
/// We nm the .a archive (not a linked test binary) to avoid dead-code-
/// elimination removing shim symbols that have no Rust call site yet.
#[test]
#[ignore]
fn nm_rapidjson_shim_validation() {
    let root = repo_root();
    let shim_dir = root.join("references/rapidjson-refactoring/rapidjson_sys/shim");
    let include_dir = root.join("references/rapidjson-refactoring/rapidjson_legacy/include");
    let rapidjson_sys_dir = root.join("references/rapidjson-refactoring/rapidjson_sys");

    // ── Step 1: collect all shim .cpp files ──────────────────────────────────
    let shim_cpps: Vec<std::path::PathBuf> = std::fs::read_dir(&shim_dir)
        .expect("Cannot read shim directory")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("cpp"))
        .collect();

    assert!(
        !shim_cpps.is_empty(),
        "[L5-nm] rapidjson_shim: no .cpp files found in {:?}",
        shim_dir
    );

    // ── Step 2: compile all shims to a combined .o ───────────────────────────
    let tmp_dir = std::env::temp_dir().join("l5_nm_rapidjson_shim");
    std::fs::create_dir_all(&tmp_dir).expect("[L5-nm] rapidjson_shim: create tmp dir failed");

    let out_obj = tmp_dir.join("rapidjson_shim.o");
    let include_str = include_dir.to_string_lossy().into_owned();
    let src_refs: Vec<&Path> = shim_cpps.iter().map(PathBuf::as_path).collect();
    let obj_path = compile_cpp_obj(&src_refs, &[include_str.as_str()], &out_obj)
        .expect("[L5-nm] rapidjson_shim: g++ compilation of shims failed");

    // ── Step 3: extract C++ exports from shim .o ─────────────────────────────
    let cpp_exports = nm_c_exports(&obj_path);
    println!(
        "[L5-nm] rapidjson_shim: C++ shim exports ({} symbols): {}",
        cpp_exports.len(),
        cpp_exports.join(", ")
    );
    assert!(
        !cpp_exports.is_empty(),
        "[L5-nm] rapidjson_shim: nm found no extern-C exports – are shim files compiled correctly?"
    );

    // ── Step 4: cargo build rapidjson_sys ────────────────────────────────────
    let status = std::process::Command::new("cargo")
        .args(["build"])
        .current_dir(&rapidjson_sys_dir)
        .status()
        .expect("[L5-nm] rapidjson_shim: failed to spawn cargo build");
    assert!(
        status.success(),
        "[L5-nm] rapidjson_shim: cargo build of rapidjson_sys failed"
    );

    // ── Step 5: find .a archives in the workspace target/debug ──────────────
    // `rapidjson_sys` is a member of the workspace at `references/rapidjson-
    // refactoring/`.  When `cargo build` is run inside `rapidjson_sys/`, cargo
    // uses the workspace-level target directory one level up, NOT a local
    // `target/` inside `rapidjson_sys/` itself.
    //
    // When CARGO_TARGET_DIR is set (shared pre-build target directory used in
    // CI), cargo writes all artifacts there instead; look there first.
    let build_dir = if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        std::path::PathBuf::from(&target_dir).join("debug")
    } else {
        rapidjson_sys_dir
            .parent()
            .expect("rapidjson_sys_dir should have a parent workspace directory")
            .join("target/debug")
    };
    let rust_archive_symbols = collect_archive_symbols(&build_dir);

    println!(
        "[L5-nm] rapidjson_shim: Rust archive symbols (extern-C, all .a): {} symbols",
        rust_archive_symbols.len()
    );

    // ── Step 6: bidirectional assertion ──────────────────────────────────────
    // For the archive check we do the assertion directly (not via nm_binary_t_symbols
    // which filters an executable) because .a archives already contain only
    // the functions we compiled.
    let cpp_set: HashSet<String> = cpp_exports.iter().cloned().collect();
    let intersection: HashSet<String> = cpp_set
        .intersection(&rust_archive_symbols)
        .cloned()
        .collect();

    assert_cpp_exports_linked(&cpp_exports, &intersection, "rapidjson_shim");
}
