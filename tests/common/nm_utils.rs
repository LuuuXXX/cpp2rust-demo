/// nm_utils: helpers for L5 symbol-level FFI validation.
///
/// The validation flow is:
///
/// ```text
/// C++ side
///   └─ g++ -c *.cpp → .o
///         └─ nm --defined-only -f posix <.o>
///               filter: type T or W, name not starting with _Z
///               → cpp_exports: {"counter_new", "counter_delete", ...}
///
/// Rust side
///   └─ cargo build → binary / static library
///         └─ nm --defined-only -f posix <artifact>
///               filter: type T, intersect with cpp_exports set
///               → rust_linked: {"counter_new", ...}
///
/// Assert: cpp_exports ⊆ rust_linked
/// ```
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

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
//  C++ side: compile + extract extern-C symbols
// ─────────────────────────────────────────────────────────────────

/// Compile one or more C++ source files to a combined object file (partial link
/// using `g++ -r`) so that a single `.o` can be nm-ed.
///
/// Returns the path to the output `.o` file, or `None` on failure.
pub fn compile_cpp_obj(srcs: &[&Path], includes: &[&str], out_path: &Path) -> Option<PathBuf> {
    if srcs.is_empty() {
        return None;
    }

    // Compile each source to an individual .o, then partial-link them together.
    let mut obj_paths: Vec<PathBuf> = Vec::new();
    for (i, src) in srcs.iter().enumerate() {
        let obj = out_path.with_extension(format!("{}.tmp.o", i));
        let mut cmd = Command::new("g++");
        cmd.args(["-c", "-fPIC", "-w"]);
        for inc in includes {
            cmd.arg(format!("-I{}", inc));
        }
        cmd.arg(src);
        cmd.arg("-o");
        cmd.arg(&obj);
        let status = cmd.status().ok()?;
        if !status.success() {
            return None;
        }
        obj_paths.push(obj);
    }

    if obj_paths.len() == 1 {
        // Only one source — rename to final path.
        std::fs::rename(&obj_paths[0], out_path).ok()?;
    } else {
        // Partial link (relocatable) to merge all objects into one.
        let mut cmd = Command::new("g++");
        cmd.args(["-r", "-o"]);
        cmd.arg(out_path);
        for obj in &obj_paths {
            cmd.arg(obj);
        }
        let status = cmd.status().ok()?;
        if !status.success() {
            return None;
        }
        // Clean up temporaries.
        for obj in &obj_paths {
            let _ = std::fs::remove_file(obj);
        }
    }

    Some(out_path.to_path_buf())
}

/// Extract extern-"C" exported symbols from a compiled `.o` file.
///
/// Returns symbols whose nm type is `T` or `W` (text section, including weak)
/// and whose name does **not** start with `_Z` (C++ mangled). This identifies
/// functions declared `extern "C"` in the source.
pub fn nm_c_exports(obj_path: &Path) -> Vec<String> {
    let output = Command::new("nm")
        .args(["--defined-only", "-f", "posix"])
        .arg(obj_path)
        .output()
        .expect("Failed to run nm on .o file");

    let text = String::from_utf8_lossy(&output.stdout);
    let mut syms: Vec<String> = parse_nm_output(&text, &['T', 'W'])
        .into_iter()
        .filter(|s| !s.starts_with("_Z") && !s.starts_with("__Z"))
        .collect();
    syms.sort();
    syms
}

/// Extract all symbols from a static archive (`.a` file) that are of type
/// `T` or `W` and do not have C++ mangled names.
pub fn nm_archive_c_exports(archive_path: &Path) -> HashSet<String> {
    let output = Command::new("nm")
        .args(["--defined-only", "-f", "posix"])
        .arg(archive_path)
        .output()
        .expect("Failed to run nm on .a file");

    let text = String::from_utf8_lossy(&output.stdout);
    parse_nm_output(&text, &['T', 'W'])
        .into_iter()
        .filter(|s| !s.starts_with("_Z") && !s.starts_with("__Z"))
        .collect()
}

// ─────────────────────────────────────────────────────────────────
//  Rust side: extract symbols from binary / archive
// ─────────────────────────────────────────────────────────────────

/// Extract all defined `T`-type symbols from a binary (executable or shared
/// library). Only symbols that appear in `filter_by` are returned, so the
/// result represents "C++ exports actually linked into the Rust binary".
pub fn nm_binary_t_symbols(bin_path: &Path, filter_by: &HashSet<String>) -> HashSet<String> {
    let output = Command::new("nm")
        .args(["--defined-only", "-f", "posix"])
        .arg(bin_path)
        .output()
        .expect("Failed to run nm on binary");

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
pub fn assert_cpp_exports_linked(cpp_exports: &[String], rust_linked: &HashSet<String>, label: &str) {
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
pub fn cargo_build_example(dir: &str, bin_name: &str) -> Option<PathBuf> {
    let status = Command::new("cargo")
        .args(["build"])
        .current_dir(dir)
        .status()
        .ok()?;

    if !status.success() {
        return None;
    }

    let bin = PathBuf::from(dir).join("target/debug").join(bin_name);
    if bin.exists() { Some(bin) } else { None }
}

/// Recursively find all `.a` static-library files under `dir`.
pub fn find_archive_files(dir: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(find_archive_files(&path));
            } else if path.extension().and_then(|e| e.to_str()) == Some("a") {
                result.push(path);
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
