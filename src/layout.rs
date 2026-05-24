use crate::error::{DemoError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FeatureLayout {
    pub project_root: PathBuf,
    pub feature: String,
    pub feature_root: PathBuf,
    pub ast_dir: PathBuf,
    pub meta_dir: PathBuf,
    pub rust_dir: PathBuf,
    pub rust_src_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapturedAstFile {
    pub json_path: PathBuf,
    pub opts_path: PathBuf,
    pub source_rel: PathBuf,
}

impl FeatureLayout {
    pub fn new(project_root: impl Into<PathBuf>, feature: impl Into<String>) -> Self {
        let project_root = project_root.into();
        let feature = feature.into();
        let feature_root = project_root.join(".cpp2rust").join(&feature);
        let ast_dir = feature_root.join("ast");
        let meta_dir = feature_root.join("meta");
        let rust_dir = feature_root.join("rust");
        let rust_src_dir = rust_dir.join("src");
        Self {
            project_root,
            feature,
            feature_root,
            ast_dir,
            meta_dir,
            rust_dir,
            rust_src_dir,
        }
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        for dir in [
            self.feature_root.as_path(),
            self.ast_dir.as_path(),
            self.meta_dir.as_path(),
            self.rust_dir.as_path(),
            self.rust_src_dir.as_path(),
        ] {
            fs::create_dir_all(dir)?;
        }
        Ok(())
    }


    pub fn build_cmd_path(&self) -> PathBuf {
        self.meta_dir.join("build_cmd.txt")
    }

    pub fn selected_files_path(&self) -> PathBuf {
        self.meta_dir.join("selected_files.json")
    }

    pub fn init_report_path(&self) -> PathBuf {
        self.meta_dir.join("init-interface-report.md")
    }

    pub fn merge_report_path(&self) -> PathBuf {
        self.meta_dir.join("merge-interface-report.md")
    }

    pub fn write_build_command(&self, build_command: &[String]) -> Result<()> {
        fs::write(self.build_cmd_path(), build_command.join(" "))?;
        Ok(())
    }

    pub fn scan_ast_files(&self) -> Result<Vec<CapturedAstFile>> {
        let mut files = Vec::new();
        if !self.ast_dir.exists() {
            return Ok(files);
        }

        for entry in WalkDir::new(&self.ast_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.into_path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let rel = path
                .strip_prefix(&self.ast_dir)
                .map_err(|_| DemoError::InvalidPath(path.clone()))?;
            let rel_string = rel.to_string_lossy();
            let source_rel = PathBuf::from(rel_string.trim_end_matches(".json"));
            let opts_path = self.ast_dir.join(format!("{}.opts", source_rel.to_string_lossy()));
            files.push(CapturedAstFile {
                json_path: path,
                opts_path,
                source_rel,
            });
        }

        files.sort_by(|a, b| a.source_rel.cmp(&b.source_rel));
        Ok(files)
    }
}

pub fn sanitize_feature_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "default".into()
    } else {
        out
    }
}

pub fn relative_display(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/test-work/layout")
            .join(format!("{name}-{}", std::process::id()));
        if root.exists() {
            std::fs::remove_dir_all(&root).unwrap();
        }
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn creates_layout_paths() {
        let dir = test_dir("paths");
        let layout = FeatureLayout::new(&dir, "demo");
        assert!(layout.feature_root.ends_with(".cpp2rust/demo"));
        assert!(layout.ast_dir.ends_with(".cpp2rust/demo/ast"));
    }

    #[test]
    fn sanitizes_feature_names() {
        assert_eq!(sanitize_feature_name("basic-types"), "basic_types");
        assert_eq!(sanitize_feature_name(""), "default");
    }

    #[test]
    fn scans_ast_files() {
        let dir = test_dir("scan");
        let layout = FeatureLayout::new(&dir, "demo");
        layout.ensure_dirs().unwrap();
        let json = layout.ast_dir.join("src/main.cpp.json");
        let opts = layout.ast_dir.join("src/main.cpp.opts");
        fs::create_dir_all(json.parent().unwrap()).unwrap();
        fs::write(&json, "{}").unwrap();
        fs::write(&opts, "-std=c++17").unwrap();
        let files = layout.scan_ast_files().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].source_rel, PathBuf::from("src/main.cpp"));
    }
}
