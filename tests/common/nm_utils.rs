//! nm_utils: helpers for L5 symbol-level FFI validation.
//!
//! The validation flow is:
//!
//! ```text
//! C++ side
//!   └─ g++ -c *.cpp → .o
//!         └─ nm --defined-only -f posix <.o>
//!               filter: type T or W, name not starting with _Z
//!               → cpp_exports: {"counter_new", "counter_delete", ...}
//!
//! Rust side
//!   └─ cargo build → binary / static library
//!         └─ nm --defined-only -f posix <artifact>
//!               filter: type T, intersect with cpp_exports set
//!               → rust_linked: {"counter_new", ...}
//!
//! Assert: cpp_exports ⊆ rust_linked
//! ```
#![allow(dead_code)]
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

// ─────────────────────────────────────────────────────────────────
//  nm / llvm-nm wrapper
// ─────────────────────────────────────────────────────────────────

/// Run `nm --defined-only -f posix` (or `llvm-nm` on Windows) on the given path.
///
/// On Windows, `llvm-nm` produces the same posix-format output as GNU `nm`,
/// making the existing `parse_nm_output` directly reusable.
fn run_nm(path: &Path) -> std::io::Result<std::process::Output> {
    #[cfg(not(windows))]
    {
        Command::new("nm")
            .args(["--defined-only", "-f", "posix"])
            .arg(path)
            .output()
    }
    #[cfg(windows)]
    {
        // Prefer llvm-nm (ships with LLVM for Windows, same posix output format).
        // Fall back to nm from MinGW/MSYS2.
        for tool in &["llvm-nm", "nm"] {
            if let Ok(out) = Command::new(tool)
                .args(["--defined-only", "--format=posix"])
                .arg(path)
                .output()
            {
                return Ok(out);
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "neither llvm-nm nor nm found; install LLVM or MinGW/MSYS2",
        ))
    }
}

// ─────────────────────────────────────────────────────────────────
//  nm output parsing
// ─────────────────────────────────────────────────────────────────

/// On macOS, `nm` prefixes all symbol names with `_`. Strip it so comparisons
/// work uniformly on both Linux and macOS.
pub fn normalize_symbol(sym: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        sym.strip_prefix('_').unwrap_or(sym).to_string()
    }
    #[cfg(not(target_os = "macos"))]
    {
        sym.to_string()
    }
}

/// Parse `nm --defined-only -f posix` output and return the set of symbol names
/// that match the given type character(s). Symbol names are normalized
/// (macOS `_` prefix stripped).
///
/// posix format: `name type address size`
fn parse_nm_output(output: &str, type_chars: &[char]) -> HashSet<String> {
    let mut result = HashSet::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.splitn(4, ' ').collect();
        if parts.len() < 2 {
            continue;
        }
        let name = parts[0];
        let type_char = parts[1].chars().next().unwrap_or(' ');
        if type_chars.contains(&type_char) {
            result.insert(normalize_symbol(name));
        }
    }
    result
}

// ─────────────────────────────────────────────────────────────────
//  Symbol classification helpers
// ─────────────────────────────────────────────────────────────────

/// Return `true` if the symbol name looks like a plain C (non-mangled) name.
///
/// **On Windows** two rules are applied:
/// - Names starting with `_` are rejected — reserved for the implementation by
///   the C and POSIX standards.  This also subsumes GCC/Clang `_Z`/`__Z`
///   mangling (MinGW) since those prefixes all begin with `_`.
/// - Names starting with `?` are rejected — MSVC mangling prefix.
/// - A small explicit denylist (`sprintf_s`, `frexpl`) covers standard C
///   functions that MSVC/clang++ with MSVC STL defines inline in system headers;
///   they appear in `.o` files but not in the `cc::Build` archive, causing
///   false-positive validation failures (observed: clang++ + MSVC STL 14.4x).
///   Extend this list if future toolchain versions add more inline CRT symbols.
///
/// **On Linux/macOS** only C++ mangling prefixes are rejected:
/// - `_Z` / `__Z` — GCC/Clang mangling
/// - `?`          — MSVC mangling (defensive, unlikely outside Windows)
fn is_c_symbol(s: &str) -> bool {
    // `main` 是程序入口点，并非 shim 导出符号。去 shim 直出（idiomatic）示例新增的
    // cpp/main.cpp 会引入唯一的非 mangled T 符号 `main`，它不应被视为需要链接进 Rust
    // 产物的 extern-C 导出。统一在此排除，避免误报。
    if s == "main" {
        return false;
    }
    #[cfg(windows)]
    {
        // `_`-prefix rejects names reserved for the implementation by the C and
        // POSIX standards; it also subsumes GCC/Clang `_Z`/`__Z` mangling (MinGW)
        // since those all start with `_`.
        // `?`-prefix is the MSVC name-mangling scheme — every MSVC-mangled name
        // begins with `?` and is distinct from the `_`-prefix category above.
        if s.starts_with('_') || s.starts_with('?') {
            return false;
        }
        // MSVC CRT functions inlined into every TU when using clang++ + MSVC STL.
        // This list was derived from observed CI failures (clang++ + MSVC STL 14.4x).
        // If future MSVC toolchain versions inline additional public-name CRT
        // functions that produce false positives, extend this list accordingly.
        const MSVC_CRT_INLINE: &[&str] = &["sprintf_s", "frexpl"];
        if MSVC_CRT_INLINE.contains(&s) {
            return false;
        }
    }
    #[cfg(not(windows))]
    if s.starts_with("_Z") || s.starts_with("__Z") || s.starts_with('?') {
        return false;
    }
    true
}

// ─────────────────────────────────────────────────────────────────
//  C++ side: compile + extract extern-C symbols
// ─────────────────────────────────────────────────────────────────

/// Compile one or more C++ source files to a combined object file (partial link
/// using `g++ -r` on Linux/macOS, `llvm-link` or `clang++` on Windows) so that
/// a single `.o` can be nm-ed.
///
/// Returns the path to the output `.o` file, or `None` on failure.
pub fn compile_cpp_obj(srcs: &[&Path], includes: &[&str], out_path: &Path) -> Option<PathBuf> {
    if srcs.is_empty() {
        return None;
    }

    // Compile each source to an individual .o, then partial-link them together.
    let mut obj_paths: Vec<PathBuf> = Vec::new();
    for (i, src) in srcs.iter().enumerate() {
        // Build a sibling filename like "<stem>.<i>.tmp.o" so we never
        // accidentally clobber the final output path.
        let stem = out_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "obj".to_string());
        let obj = out_path.with_file_name(format!("{}.{}.tmp.o", stem, i));
        let status = cxx_compile_obj(src, includes, &obj)?;
        if !status {
            return None;
        }
        obj_paths.push(obj);
    }

    if obj_paths.len() == 1 {
        // Only one source — rename to final path.
        std::fs::rename(&obj_paths[0], out_path).ok()?;
    } else {
        // Partial link (relocatable) to merge all objects into one.
        let status = cxx_partial_link(&obj_paths, out_path)?;
        if !status {
            return None;
        }
        // Clean up temporaries.
        for obj in &obj_paths {
            let _ = std::fs::remove_file(obj);
        }
    }

    Some(out_path.to_path_buf())
}

/// Compile a single C++ source to a `.o` file. Returns `Some(true)` on success.
fn cxx_compile_obj(src: &Path, includes: &[&str], obj: &Path) -> Option<bool> {
    #[cfg(not(windows))]
    {
        let mut cmd = Command::new("g++");
        cmd.args(["-c", "-fPIC", "-w"]);
        for inc in includes {
            cmd.arg(format!("-I{}", inc));
        }
        cmd.arg(src).arg("-o").arg(obj);
        Some(cmd.status().ok()?.success())
    }
    #[cfg(windows)]
    {
        // On Windows: prefer clang++ (compatible output), fall back to g++ (MinGW)
        // Note: -fPIC is not supported on Windows PE/COFF targets.
        // -D_ALLOW_COMPILER_AND_STL_VERSION_MISMATCH suppresses the STL1000 error
        // that MSVC STL 14.44+ raises when used with clang < 19.
        for (compiler, extra_flags) in &[
            (
                "clang++",
                vec!["-D_ALLOW_COMPILER_AND_STL_VERSION_MISMATCH"],
            ),
            ("g++", vec![]),
        ] {
            let mut cmd = Command::new(compiler);
            cmd.args(["-c", "-w"]);
            cmd.args(extra_flags);
            for inc in includes {
                cmd.arg(format!("-I{}", inc));
            }
            cmd.arg(src).arg("-o").arg(obj);
            if let Ok(status) = cmd.status() {
                if status.success() {
                    return Some(true);
                }
            }
        }
        None
    }
}

/// Partial-link (relocatable) multiple object files into one. Returns `Some(true)` on success.
fn cxx_partial_link(objs: &[PathBuf], out: &Path) -> Option<bool> {
    #[cfg(not(windows))]
    {
        let mut cmd = Command::new("g++");
        cmd.args(["-r", "-o"]).arg(out);
        for obj in objs {
            cmd.arg(obj);
        }
        Some(cmd.status().ok()?.success())
    }
    #[cfg(windows)]
    {
        // On Windows MSVC, clang++ routes to link.exe which does not support -r.
        // Use llvm-ar to create a static archive instead (llvm-nm can read it).
        // Fall back to g++ -r for MinGW environments where it works.
        {
            let mut cmd = Command::new("llvm-ar");
            cmd.arg("rcs").arg(out);
            for obj in objs {
                cmd.arg(obj);
            }
            if let Ok(status) = cmd.status() {
                if status.success() {
                    return Some(true);
                }
            }
        }
        {
            let mut cmd = Command::new("g++");
            cmd.args(["-r", "-o"]).arg(out);
            for obj in objs {
                cmd.arg(obj);
            }
            if let Ok(status) = cmd.status() {
                if status.success() {
                    return Some(true);
                }
            }
        }
        None
    }
}

/// Extract extern-"C" exported symbols from a compiled `.o` file.
///
/// Returns symbols whose nm type is `T` or `W` (text section, including weak)
/// and whose name does **not** start with a C++ mangling prefix:
/// - `_Z` / `__Z` — GCC/Clang (Linux, macOS, MinGW) mangling
/// - `?`          — MSVC mangling (Windows); all MSVC-mangled names begin with `?`
///
/// This identifies functions declared `extern "C"` in the source.
pub fn nm_c_exports(obj_path: &Path) -> Vec<String> {
    let output = run_nm(obj_path).expect("Failed to run nm/llvm-nm on .o file");

    let text = String::from_utf8_lossy(&output.stdout);
    let mut syms: Vec<String> = parse_nm_output(&text, &['T', 'W'])
        .into_iter()
        .filter(|s| is_c_symbol(s))
        .collect();
    syms.sort();
    syms
}

/// Extract all symbols from a static archive (`.a` or `.lib` file) that are of type
/// `T` or `W` and do not have C++ mangled names (GCC/Clang `_Z`/`__Z` prefix or
/// MSVC `?` prefix).
pub fn nm_archive_c_exports(archive_path: &Path) -> HashSet<String> {
    let output = run_nm(archive_path).expect("Failed to run nm/llvm-nm on archive file");

    let text = String::from_utf8_lossy(&output.stdout);
    parse_nm_output(&text, &['T', 'W'])
        .into_iter()
        .filter(|s| is_c_symbol(s))
        .collect()
}

// ─────────────────────────────────────────────────────────────────
//  Rust side: extract symbols from binary / archive
// ─────────────────────────────────────────────────────────────────

/// Extract all defined `T`-type symbols from a binary (executable or shared
/// library). Only symbols that appear in `filter_by` are returned, so the
/// result represents "C++ exports actually linked into the Rust binary".
pub fn nm_binary_t_symbols(bin_path: &Path, filter_by: &HashSet<String>) -> HashSet<String> {
    let output = run_nm(bin_path).expect("Failed to run nm/llvm-nm on binary");

    let text = String::from_utf8_lossy(&output.stdout);
    parse_nm_output(&text, &['T', 'W'])
        .into_iter()
        .filter(|s| filter_by.contains(s.as_str()))
        .collect()
}

// ─────────────────────────────────────────────────────────────────
//  Assertion + reporting
// ─────────────────────────────────────────────────────────────────

/// Assert that every symbol in `cpp_exports` is present in `rust_linked`.
/// Always prints a report; panics with a diff when any are missing.
pub fn assert_cpp_exports_linked(
    cpp_exports: &[String],
    rust_linked: &HashSet<String>,
    label: &str,
) {
    let mut missing: Vec<&str> = cpp_exports
        .iter()
        .filter(|s| !rust_linked.contains(s.as_str()))
        .map(String::as_str)
        .collect();
    missing.sort();

    let present: Vec<&str> = cpp_exports
        .iter()
        .filter(|s| rust_linked.contains(s.as_str()))
        .map(String::as_str)
        .collect();

    println!(
        "[L5-nm] {label}\n  C++ .o exports (extern \"C\"): [{syms}]  [{n} symbols]",
        label = label,
        syms = cpp_exports.join(", "),
        n = cpp_exports.len()
    );
    println!(
        "  Rust binary linked (intersection): [{syms}]  [{n} symbols]",
        syms = present.join(", "),
        n = present.len()
    );

    if missing.is_empty() {
        println!("  Missing in Rust: (none) ✓\n");
    } else {
        println!("  Missing in Rust: {}\n", missing.join(", "));
        panic!(
            "[L5-nm] {label}: {n} C++ extern-C export(s) not found in Rust binary.\n\
             Missing: {missing}",
            label = label,
            n = missing.len(),
            missing = missing.join(", ")
        );
    }
}

// ─────────────────────────────────────────────────────────────────
//  Build helpers
// ─────────────────────────────────────────────────────────────────

/// Run `cargo build` in `dir` and return the path to the compiled binary named
/// `bin_name` (the `package.name` in that crate's Cargo.toml).
/// Returns `None` if the build fails or the binary is not found.
///
/// When the `CARGO_TARGET_DIR` environment variable is set (e.g. a shared
/// target directory used by the pre-build CI step), the binary is looked up
/// there first before falling back to the crate-local `target/debug/`.
pub fn cargo_build_example(dir: &str, bin_name: &str) -> Option<PathBuf> {
    let status = Command::new("cargo")
        .args(["build"])
        .current_dir(dir)
        .status()
        .ok()?;

    if !status.success() {
        return None;
    }

    #[cfg(windows)]
    let bin_name_with_ext = format!("{}.exe", bin_name);
    #[cfg(not(windows))]
    let bin_name_with_ext = bin_name.to_string();

    // Prefer the shared target directory when CARGO_TARGET_DIR is set.
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        let bin = PathBuf::from(&target_dir)
            .join("debug")
            .join(&bin_name_with_ext);
        if bin.exists() {
            return Some(bin);
        }
    }

    let bin = PathBuf::from(dir)
        .join("target/debug")
        .join(&bin_name_with_ext);
    if bin.exists() {
        Some(bin)
    } else {
        None
    }
}

/// Recursively find all static-library files (`.a` on Unix, `.a` and `.lib` on Windows) under `dir`.
pub fn find_archive_files(dir: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(find_archive_files(&path));
            } else {
                let ext = path.extension().and_then(|e| e.to_str());
                #[cfg(windows)]
                let is_archive = ext == Some("a") || ext == Some("lib");
                #[cfg(not(windows))]
                let is_archive = ext == Some("a");
                if is_archive {
                    result.push(path);
                }
            }
        }
    }
    result
}

/// Collect all extern-C symbols from every `.a` archive found under `build_dir`.
pub fn collect_archive_symbols(build_dir: &Path) -> HashSet<String> {
    let mut all = HashSet::new();
    for archive in find_archive_files(build_dir) {
        all.extend(nm_archive_c_exports(&archive));
    }
    all
}
