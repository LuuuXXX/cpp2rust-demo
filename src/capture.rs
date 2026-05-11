use crate::error::Result;
use anyhow::anyhow;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Build `hook/libhook.so`.
pub fn build_hook() -> Result<PathBuf> {
    let hook_dir = hook_dir()?;
    let so = hook_dir.join("libhook.so");
    if so.exists() {
        let metadata = std::fs::metadata(&so)
            .map_err(|e| anyhow!("stat {}: {}", so.display(), e))?;
        if metadata.len() > 0 {
            return Ok(so);
        }
    }

    let status = Command::new("make")
        .current_dir(&hook_dir)
        .arg("-s")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| anyhow!("failed to run make in {}: {}", hook_dir.display(), e))?;

    if !status.success() {
        return Err(anyhow!("make failed in {}", hook_dir.display()));
    }
    if !so.exists() {
        return Err(anyhow!("libhook.so not found after build at {}", so.display()));
    }
    Ok(so)
}

/// Execute command with LD_PRELOAD hook enabled.
pub fn run_with_hook(
    build_dir: &Path,
    cmd: &[String],
    project_root: &Path,
    feature_root: &Path,
    hook_so: &Path,
) -> Result<()> {
    if cmd.is_empty() {
        return Err(anyhow!("capture command is empty"));
    }

    let abs_project_root = project_root
        .canonicalize()
        .map_err(|e| anyhow!("canonicalize {}: {}", project_root.display(), e))?;
    let abs_feature_root = feature_root
        .canonicalize()
        .map_err(|e| anyhow!("canonicalize {}: {}", feature_root.display(), e))?;
    let abs_hook = hook_so
        .canonicalize()
        .map_err(|e| anyhow!("canonicalize {}: {}", hook_so.display(), e))?;

    let status = Command::new(&cmd[0])
        .args(&cmd[1..])
        .current_dir(build_dir)
        .env("LD_PRELOAD", &abs_hook)
        .env("CPP2RUST_PROJECT_ROOT", &abs_project_root)
        .env("CPP2RUST_FEATURE_ROOT", &abs_feature_root)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| anyhow!("failed to spawn '{}': {}", cmd[0], e))?;

    if !status.success() {
        return Err(anyhow!(
            "capture command failed with exit code {}",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

/// Load headers captured by hook: `.cpp2rust/<feature>/meta/captured_headers.list`.
pub fn load_captured_headers(feature_root: &Path) -> Result<Vec<PathBuf>> {
    let list_path = feature_root.join("meta").join("captured_headers.list");
    if !list_path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&list_path)
        .map_err(|e| anyhow!("read {}: {}", list_path.display(), e))?;
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        out.push(PathBuf::from(line));
    }
    out.sort();
    out.dedup();
    Ok(out)
}

/// Parse selected `*.cpp2rust` middleware files and infer project header files
/// from preprocessor line markers.
pub fn infer_headers_from_cpp2rust_files(
    cpp2rust_files: &[PathBuf],
    project_root: &Path,
) -> Result<Vec<PathBuf>> {
    let mut headers: HashSet<PathBuf> = HashSet::new();

    for file in cpp2rust_files {
        let content = std::fs::read_to_string(file)
            .map_err(|e| anyhow!("read {}: {}", file.display(), e))?;
        for line in content.lines() {
            let Some(raw_path) = parse_line_marker_path(line) else {
                continue;
            };
            if !has_header_ext(&raw_path) {
                continue;
            }

            let path = normalize_marker_path(&raw_path, project_root);
            if !path.exists() || !path.starts_with(project_root) {
                continue;
            }
            headers.insert(path);
        }
    }

    let mut out: Vec<PathBuf> = headers.into_iter().collect();
    out.sort();
    Ok(out)
}

fn parse_line_marker_path(line: &str) -> Option<String> {
    // GCC/Clang preprocessor markers:
    //   # 1 "/abs/path/header.hpp" 1
    //   #line 12 "/abs/path/header.hpp"
    let quote_start = line.find('"')?;
    let rest = &line[quote_start + 1..];
    let quote_end = rest.find('"')?;
    Some(rest[..quote_end].to_string())
}

fn normalize_marker_path(raw: &str, project_root: &Path) -> PathBuf {
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    }
}

fn has_header_ext(path: &str) -> bool {
    let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) else {
        return false;
    };
    matches!(ext, "h" | "hpp" | "hh" | "hxx" | "H" | "HPP" | "HH" | "HXX" | "h++" | "H++")
}

fn hook_dir() -> Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("hook");
            if candidate.join("Makefile").exists() {
                return Ok(candidate);
            }
            if let Some(workspace) = parent.parent().and_then(|p| p.parent()) {
                let candidate = workspace.join("hook");
                if candidate.join("Makefile").exists() {
                    return Ok(candidate);
                }
            }
        }
    }

    let cwd_candidate = std::env::current_dir()
        .map_err(|e| anyhow!("current_dir: {}", e))?
        .join("hook");
    if cwd_candidate.join("Makefile").exists() {
        return Ok(cwd_candidate);
    }

    Err(anyhow!(
        "hook/ directory with Makefile not found (searched near binary and cwd)"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn infer_headers_from_cpp2rust_files_collects_project_headers() {
        let tmp = TempDir::new().unwrap();
        let project_root = tmp.path();
        let feature_cpp_dir = project_root.join(".cpp2rust/default/cpp");
        std::fs::create_dir_all(&feature_cpp_dir).unwrap();

        let header = project_root.join("include").join("demo.hpp");
        std::fs::create_dir_all(header.parent().unwrap()).unwrap();
        std::fs::write(&header, "int add(int a, int b);").unwrap();

        let middleware = feature_cpp_dir.join("main.cpp2rust");
        std::fs::write(
            &middleware,
            format!(
                "# 1 \"{}\" 1\nint add(int a, int b);\n# 1 \"/usr/include/stdio.h\" 1\n",
                header.display()
            ),
        )
        .unwrap();

        let inferred = infer_headers_from_cpp2rust_files(&[middleware], project_root).unwrap();
        assert_eq!(inferred, vec![header]);
    }
}
