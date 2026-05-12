use crate::error::Result;
use anyhow::anyhow;
use std::path::{Path, PathBuf};

/// Locate the project root by searching for `.cpp2rust/` upward from `start`.
/// Falls back to `start` itself if not found.
pub fn find_project_root(start: &Path) -> PathBuf {
    let mut cur = start.to_path_buf();
    loop {
        if cur.join(".cpp2rust").is_dir() {
            return cur;
        }
        match cur.parent() {
            Some(p) => cur = p.to_path_buf(),
            None => return start.to_path_buf(),
        }
    }
}

/// Layout of a single feature directory under `.cpp2rust/<feature>/`.
pub struct FeatureLayout {
    #[allow(dead_code)]
    pub project_root: PathBuf,
    #[allow(dead_code)]
    pub feature_name: String,
    /// `.cpp2rust/<feature>/`
    pub feature_root: PathBuf,
    /// `.cpp2rust/<feature>/cpp/`   – captured preprocessed middleware files
    pub cpp_dir: PathBuf,
    /// `.cpp2rust/<feature>/ast/`   – raw clang AST JSON files
    pub ast_dir: PathBuf,
    /// `.cpp2rust/<feature>/rust/`  – generated Rust project
    pub rust_dir: PathBuf,
    /// `.cpp2rust/<feature>/meta/`  – metadata (selected files list, link name, etc.)
    pub meta_dir: PathBuf,
}

impl FeatureLayout {
    pub fn new(project_root: PathBuf, feature_name: &str) -> Self {
        let feature_root = project_root.join(".cpp2rust").join(feature_name);
        Self {
            cpp_dir: feature_root.join("cpp"),
            ast_dir: feature_root.join("ast"),
            rust_dir: feature_root.join("rust"),
            meta_dir: feature_root.join("meta"),
            feature_root,
            project_root,
            feature_name: feature_name.to_string(),
        }
    }

    /// Create all required directories.
    pub fn create_dirs(&self) -> Result<()> {
        for dir in [&self.cpp_dir, &self.ast_dir, &self.rust_dir, &self.meta_dir] {
            std::fs::create_dir_all(dir)
                .map_err(|e| anyhow!("create dir {}: {}", dir.display(), e))?;
        }
        Ok(())
    }

    /// Write `meta/headers.json` – the list of selected middleware files and link name.
    pub fn save_meta(&self, selected_files: &[PathBuf], link_name: &str) -> Result<()> {
        #[derive(serde::Serialize)]
        struct Meta<'a> {
            link_name: &'a str,
            selected_files: Vec<String>,
        }
        let meta = Meta {
            link_name,
            selected_files: selected_files
                .iter()
                .map(|p| p.display().to_string())
                .collect(),
        };
        let json =
            serde_json::to_string_pretty(&meta).map_err(|e| anyhow!("serialize meta: {}", e))?;
        let path = self.meta_dir.join("headers.json");
        std::fs::write(&path, json).map_err(|e| anyhow!("write {}: {}", path.display(), e))
    }

    /// Write `meta/selected_files.json` – middleware files selected by the user.
    pub fn save_selected_files(&self, middleware_files: &[PathBuf]) -> Result<()> {
        let list: Vec<String> = middleware_files
            .iter()
            .map(|p| p.display().to_string())
            .collect();
        let json = serde_json::to_string_pretty(&list)
            .map_err(|e| anyhow!("serialize selected_files: {}", e))?;
        let path = self.meta_dir.join("selected_files.json");
        std::fs::write(&path, json).map_err(|e| anyhow!("write {}: {}", path.display(), e))
    }

    /// Write `meta/build_cmd.txt` – the original build command passed to `init`.
    pub fn save_build_cmd(&self, build_cmd: &[String]) -> Result<()> {
        let path = self.meta_dir.join("build_cmd.txt");
        let content = build_cmd.join(" ");
        std::fs::write(&path, content).map_err(|e| anyhow!("write {}: {}", path.display(), e))
    }

    /// Load `meta/headers.json`.
    pub fn load_meta(&self) -> Result<(String, Vec<PathBuf>)> {
        #[derive(serde::Deserialize)]
        struct Meta {
            link_name: String,
            selected_files: Vec<String>,
        }
        let path = self.meta_dir.join("headers.json");
        let json = std::fs::read_to_string(&path)
            .map_err(|e| anyhow!("read {}: {}", path.display(), e))?;
        let meta: Meta = serde_json::from_str(&json).map_err(|e| anyhow!("parse meta: {}", e))?;
        Ok((
            meta.link_name,
            meta.selected_files.iter().map(PathBuf::from).collect(),
        ))
    }
}

/// Scan `.cpp2rust/<feature>/cpp/` for all captured `*.cpp2rust` middleware files.
pub fn scan_cpp2rust_files(cpp_dir: &Path) -> Result<Vec<PathBuf>> {
    if !cpp_dir.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    visit_dir(cpp_dir, &mut out)?;
    out.sort();
    Ok(out)
}

fn visit_dir(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir).map_err(|e| anyhow!("read_dir {}: {}", dir.display(), e))? {
        let entry = entry.map_err(|e| anyhow!("read entry: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            visit_dir(&path, out)?;
        } else if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".cpp2rust"))
        {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn find_project_root_in_current_dir() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".cpp2rust")).unwrap();
        assert_eq!(find_project_root(tmp.path()), tmp.path());
    }

    #[test]
    fn find_project_root_in_parent() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".cpp2rust")).unwrap();
        let sub = tmp.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        assert_eq!(find_project_root(&sub), tmp.path());
    }

    #[test]
    fn find_project_root_fallback() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        assert_eq!(find_project_root(&sub), sub);
    }

    #[test]
    fn feature_layout_create_dirs() {
        let tmp = TempDir::new().unwrap();
        let layout = FeatureLayout::new(tmp.path().to_path_buf(), "default");
        layout.create_dirs().unwrap();
        assert!(layout.cpp_dir.exists());
        assert!(layout.ast_dir.exists());
        assert!(layout.rust_dir.exists());
        assert!(layout.meta_dir.exists());
    }

    #[test]
    fn save_and_load_meta() {
        let tmp = TempDir::new().unwrap();
        let layout = FeatureLayout::new(tmp.path().to_path_buf(), "default");
        layout.create_dirs().unwrap();
        let headers = vec![PathBuf::from("/tmp/mylib.hpp")];
        layout.save_meta(&headers, "mylib").unwrap();
        let (link, loaded) = layout.load_meta().unwrap();
        assert_eq!(link, "mylib");
        assert_eq!(loaded, headers);
    }

    #[test]
    fn save_selected_files_writes_json() {
        let tmp = TempDir::new().unwrap();
        let layout = FeatureLayout::new(tmp.path().to_path_buf(), "default");
        layout.create_dirs().unwrap();
        let headers = vec![
            PathBuf::from("/capture/foo.cpp.cpp2rust"),
            PathBuf::from("/capture/bar.cc.cpp2rust"),
        ];
        layout.save_selected_files(&headers).unwrap();
        let content = std::fs::read_to_string(layout.meta_dir.join("selected_files.json")).unwrap();
        assert!(content.contains("foo.cpp.cpp2rust"));
        assert!(content.contains("bar.cc.cpp2rust"));
    }

    #[test]
    fn save_build_cmd_writes_meta_file() {
        let tmp = TempDir::new().unwrap();
        let layout = FeatureLayout::new(tmp.path().to_path_buf(), "default");
        layout.create_dirs().unwrap();
        layout
            .save_build_cmd(&["make".to_string(), "-j4".to_string()])
            .unwrap();
        let content = std::fs::read_to_string(layout.meta_dir.join("build_cmd.txt")).unwrap();
        assert_eq!(content, "make -j4");
    }

    #[test]
    fn scan_cpp2rust_files_finds_files() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("x")).unwrap();
        std::fs::write(tmp.path().join("a.cpp.cpp2rust"), "").unwrap();
        std::fs::write(tmp.path().join("x/b.cc.cpp2rust"), "").unwrap();
        std::fs::write(tmp.path().join("x/c.cpp"), "").unwrap();
        std::fs::write(tmp.path().join("x/d.cpp.cpp2rust.opts"), "").unwrap();
        let files = scan_cpp2rust_files(tmp.path()).unwrap();
        assert_eq!(files.len(), 2);
    }
}
