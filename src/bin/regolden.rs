use std::fs;
use std::path::Path;

fn main() {
    let only: Vec<String> = std::env::args().skip(1).collect();

    let examples_dir = "examples";
    let entries = fs::read_dir(examples_dir).unwrap();
    let mut example_dirs: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
        .collect();
    example_dirs.sort();

    for dir in &example_dirs {
        if !only.is_empty() {
            let matches = only.iter().any(|p| dir.contains(p.as_str()));
            if !matches {
                continue;
            }
        }
        let example_dir = format!("examples/{}", dir);
        let scaffold_path = format!("{}/rust_hicc/src/lib_scaffold.rs", example_dir);

        if !Path::new(&scaffold_path).exists() {
            continue;
        }

        println!("=== {} ===", dir);
        let generated = run_tool_on(&example_dir);
        if generated.is_empty() {
            eprintln!("  (no output from tool, skipping {})", dir);
            continue;
        }

        let blocks = extract_hicc_blocks(&generated);
        if blocks.is_empty() {
            eprintln!("  (no hicc blocks extracted, skipping {})", dir);
            continue;
        }

        fs::write(&scaffold_path, &blocks).unwrap();
        println!("  wrote {}", scaffold_path);
    }
}

fn run_tool_on(example_dir: &str) -> String {
    let cpp_dir = format!("{}/cpp", example_dir);
    let cpp_file = find_cpp_file(&cpp_dir);
    let cpp_file = match cpp_file {
        Some(p) => p,
        None => {
            eprintln!("  no .cpp file in {}", cpp_dir);
            return String::new();
        }
    };

    let unit_name = cpp_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unit")
        .to_string();

    let tmp_dir = std::env::temp_dir().join(format!("cpp2rust_regolden_{}", unit_name));
    std::fs::create_dir_all(&tmp_dir).ok();
    let preprocessed = tmp_dir.join(format!("{}.cpp2rust", unit_name));

    if !run_preprocess(&cpp_file, &preprocessed) {
        eprintln!("  preprocess failed for {}", cpp_file.display());
        return String::new();
    }

    let ast = match cpp2rust_demo::ast_parser::parse_preprocessed(&preprocessed) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("  AST parse failed: {}", e);
            return String::new();
        }
    };

    let (system_includes, project_header) =
        cpp2rust_demo::extractor::read_source_includes(&cpp_file);

    let spec = cpp2rust_demo::extractor::extract(
        &ast,
        &unit_name,
        &system_includes,
        project_header.as_deref(),
    );

    cpp2rust_demo::generator::hicc_codegen::generate(&spec)
}

fn find_cpp_file(cpp_dir: &str) -> Option<std::path::PathBuf> {
    let entries = fs::read_dir(cpp_dir).ok()?;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("cpp") {
            return Some(path);
        }
    }
    None
}

fn run_preprocess(cpp_file: &Path, output: &Path) -> bool {
    let try_cxx = |compiler: &str| -> bool {
        std::process::Command::new(compiler)
            .args(["-E", "-C", "-o"])
            .arg(output)
            .arg(cpp_file)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };
    try_cxx("clang++") || try_cxx("g++")
}

fn extract_hicc_blocks(raw: &str) -> String {
    let blocks = cpp2rust_demo::merger::block_parser::extract_block_texts(raw);
    let mut result = String::new();
    for block in blocks {
        result.push_str(&block);
        result.push_str("\n\n");
    }
    result.trim_end().to_string() + "\n"
}
