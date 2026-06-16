//! 编译元数据落盘与解析（方案 A：让生成的 `build.rs` 自动得知第三方库的头/实现路径）。
//!
//! `init` 阶段的 LD_PRELOAD hook（`hook/hook.cpp`）在拦截 `g++`/`clang++` 调用时，
//! 除了写出预处理结果 `<unit>.cpp2rust`，还会把抽取出的编译选项（`-I` / `-isystem` /
//! `-iquote` / `-std=…` 等）以带引号的形式保存到同名 `<unit>.cpp2rust.opts` 文件。
//!
//! 本模块负责：
//! 1. 解析这些 `.opts` 文件，还原 **include 路径** 与 **C++ 标准**；
//! 2. 由 `.cpp2rust` 路径反推 **底层实现 `.cpp`**（被 FFI 绑定的符号定义所在单元）；
//! 3. 聚合为 [`BuildMeta`]，供 [`crate::generator::project_generator::write_build_rs`]
//!    在生成的 `build.rs` 中注入 `cc_build.std(...)` / `cc_build.include(...)` /
//!    `cc_build.file(...)`，使端到端 `cargo check` / `cargo test` 无需外部脚本就地改写
//!    `build.rs` 即可编译并链接通过。
//!
//! 当没有任何 `.opts` 元数据（例如黄金测试 / `gen-verify` 直接调用生成器）时，
//! [`BuildMeta`] 为空，`write_build_rs` 退化为最小化输出，产物逐字节不变。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 由捕获的 `.opts` 与选定单元聚合而成的编译元数据。
///
/// 所有路径在聚合时已尽量规整为绝对路径（`include_dirs` / `impl_sources`），
/// 以便在生成项目目录（`.cpp2rust/<feature>/rust/`）下构建时正确解析。
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildMeta {
    /// C++ 标准（不含 `-std=` 前缀），例如 `"c++17"`。取自首个声明该选项的单元。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpp_std: Option<String>,
    /// include 搜索路径（`-I` / `-isystem` / `-iquote`），按首次出现顺序去重。
    #[serde(default)]
    pub include_dirs: Vec<String>,
    /// 底层实现 `.cpp` 源文件（被绑定符号的定义所在），按选定顺序去重。
    #[serde(default)]
    pub impl_sources: Vec<String>,
}

impl BuildMeta {
    /// 是否不含任何可注入的元数据。为空时生成器退化为最小化 `build.rs`。
    pub fn is_empty(&self) -> bool {
        self.cpp_std.is_none() && self.include_dirs.is_empty() && self.impl_sources.is_empty()
    }

    /// 由选定的 `.cpp2rust` 文件列表聚合编译元数据。
    ///
    /// - `selected`：用户选定的 `.cpp2rust` 文件绝对路径（与 `init` 第一遍解析一致）。
    /// - `c_dir`：`.cpp2rust/<feature>/c/`，用于从 `.cpp2rust` 路径反推相对项目根的源路径。
    /// - `project_root`：工程根目录，用于还原原始 `.cpp` 绝对路径。
    ///
    /// 对每个选定文件，读取其同名 `<file>.opts`（即 `<file>` 追加 `.opts`），抽取
    /// include 路径与 `-std`；同时把对应的原始 `.cpp` 作为实现源加入。缺失或无法解析的
    /// `.opts` 会被静默跳过（退化为不注入该单元的选项），不影响其余单元。
    pub fn collect(selected: &[PathBuf], c_dir: &Path, project_root: &Path) -> Self {
        let mut meta = BuildMeta::default();

        for cpp2rust_path in selected {
            // ① 还原原始 .cpp 路径（与 init::first_pass_parse 的反推规则一致）。
            let original_cpp = original_cpp_path(cpp2rust_path, c_dir, project_root);
            if let Some(src) = canonicalize_lossy(&original_cpp) {
                push_unique(&mut meta.impl_sources, src);
            }

            // ② 解析 <file>.opts 中的 include 路径与 -std。
            let opts_path = append_ext(cpp2rust_path, "opts");
            let Ok(content) = std::fs::read_to_string(&opts_path) else {
                continue;
            };
            let (std_opt, includes) = parse_opts(&content);
            if meta.cpp_std.is_none() {
                if let Some(s) = std_opt {
                    meta.cpp_std = Some(s);
                }
            }
            for inc in includes {
                // include 路径多为绝对路径；相对路径则按项目根解析，规整失败保留原值。
                let resolved = canonicalize_lossy(Path::new(&inc))
                    .or_else(|| canonicalize_lossy(&project_root.join(&inc)))
                    .unwrap_or(inc);
                push_unique(&mut meta.include_dirs, resolved);
            }
        }

        meta
    }
}

/// 从 `.cpp2rust` 路径反推原始 `.cpp` 的绝对路径。
///
/// hook 命名规则：`<c_dir>/<relative_from_project_root>.cpp2rust`，
/// 例：`<c_dir>/src/foo.cpp.cpp2rust` → `<project_root>/src/foo.cpp`。
fn original_cpp_path(cpp2rust_path: &Path, c_dir: &Path, project_root: &Path) -> PathBuf {
    let rel = cpp2rust_path.strip_prefix(c_dir).unwrap_or(cpp2rust_path);
    let rel_str = rel.to_string_lossy();
    let cpp_rel = rel_str
        .strip_suffix(".cpp2rust")
        .unwrap_or(&rel_str)
        .to_string();
    project_root.join(cpp_rel)
}

/// 在路径末尾追加扩展名（`foo.cpp2rust` → `foo.cpp2rust.opts`）。
fn append_ext(path: &Path, ext: &str) -> PathBuf {
    let mut s = path.as_os_str().to_os_string();
    s.push(".");
    s.push(ext);
    PathBuf::from(s)
}

/// 规整为绝对路径字符串；不存在或失败时返回 `None`（include）或回退由调用方处理。
fn canonicalize_lossy(path: &Path) -> Option<String> {
    std::fs::canonicalize(path)
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

/// 顺序去重地向 `vec` 追加 `value`。
fn push_unique(vec: &mut Vec<String>, value: String) {
    if !vec.iter().any(|v| v == &value) {
        vec.push(value);
    }
}

/// 解析 `.opts` 文件内容，返回 `(c++ 标准, include 路径列表)`。
///
/// `.opts` 由 hook 的 `save_options` 写出，每个 token 以双引号包裹、空格分隔，例如：
/// `"-std=c++17" "-I/abs/inc" "-isystem" "/abs/sys" "-Dfoo=bar" `。
/// 支持 `-I<dir>` 合并形式与 `-I <dir>` / `-isystem <dir>` / `-iquote <dir>` 分离形式。
/// `-D` / `-U` / `-fshort-enums` 等不影响 include 解析的选项被忽略。
pub fn parse_opts(content: &str) -> (Option<String>, Vec<String>) {
    let tokens = tokenize(content);
    let mut cpp_std: Option<String> = None;
    let mut includes: Vec<String> = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        let tok = &tokens[i];
        if let Some(std_val) = tok.strip_prefix("-std=") {
            if cpp_std.is_none() && !std_val.is_empty() {
                cpp_std = Some(std_val.to_string());
            }
        } else if let Some(dir) = tok.strip_prefix("-I") {
            if dir.is_empty() {
                // 分离形式：-I <dir>
                if let Some(next) = tokens.get(i + 1) {
                    push_unique(&mut includes, next.clone());
                    i += 1;
                }
            } else {
                // 合并形式：-I<dir>
                push_unique(&mut includes, dir.to_string());
            }
        } else if tok == "-isystem" || tok == "-iquote" {
            if let Some(next) = tokens.get(i + 1) {
                push_unique(&mut includes, next.clone());
                i += 1;
            }
        }
        i += 1;
    }

    (cpp_std, includes)
}

/// 从 `.opts` 内容中抽取所有被双引号包裹的 token。
fn tokenize(content: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = content.chars();
    while let Some(c) = chars.next() {
        if c == '"' {
            let mut cur = String::new();
            for ch in chars.by_ref() {
                if ch == '"' {
                    break;
                }
                cur.push(ch);
            }
            tokens.push(cur);
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_extracts_quoted_tokens() {
        let toks = tokenize("\"-std=c++17\" \"-I/abs/inc\" \"-isystem\" \"/sys\" ");
        assert_eq!(toks, vec!["-std=c++17", "-I/abs/inc", "-isystem", "/sys"]);
    }

    #[test]
    fn parse_opts_combined_include_and_std() {
        let (std, inc) = parse_opts("\"-std=c++17\" \"-I/a/inc\" \"-I/b/inc\" ");
        assert_eq!(std, Some("c++17".to_string()));
        assert_eq!(inc, vec!["/a/inc", "/b/inc"]);
    }

    #[test]
    fn parse_opts_separated_include_forms() {
        let (std, inc) =
            parse_opts("\"-I\" \"/a\" \"-isystem\" \"/sys\" \"-iquote\" \"/q\" \"-Dfoo\" ");
        assert_eq!(std, None);
        assert_eq!(inc, vec!["/a", "/sys", "/q"]);
    }

    #[test]
    fn parse_opts_dedups_includes_preserving_order() {
        let (_std, inc) = parse_opts("\"-I/a\" \"-I/b\" \"-I/a\" ");
        assert_eq!(inc, vec!["/a", "/b"]);
    }

    #[test]
    fn parse_opts_first_std_wins() {
        let (std, _inc) = parse_opts("\"-std=c++11\" \"-std=c++17\" ");
        assert_eq!(std, Some("c++11".to_string()));
    }

    #[test]
    fn parse_opts_empty_is_empty() {
        let (std, inc) = parse_opts("");
        assert_eq!(std, None);
        assert!(inc.is_empty());
    }

    #[test]
    fn build_meta_default_is_empty() {
        assert!(BuildMeta::default().is_empty());
    }

    #[test]
    fn build_meta_non_empty_when_any_field_set() {
        let m = BuildMeta {
            cpp_std: Some("c++17".to_string()),
            ..Default::default()
        };
        assert!(!m.is_empty());
    }

    #[test]
    fn append_ext_appends_dot_suffix() {
        let p = append_ext(Path::new("/x/foo.cpp.cpp2rust"), "opts");
        assert_eq!(p, PathBuf::from("/x/foo.cpp.cpp2rust.opts"));
    }

    #[test]
    fn original_cpp_path_reverses_hook_naming() {
        let c_dir = Path::new("/proj/.cpp2rust/feat/c");
        let root = Path::new("/proj");
        let cpp2rust = Path::new("/proj/.cpp2rust/feat/c/src/foo.cpp.cpp2rust");
        let got = original_cpp_path(cpp2rust, c_dir, root);
        assert_eq!(got, PathBuf::from("/proj/src/foo.cpp"));
    }

    #[test]
    fn collect_aggregates_opts_and_sources() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let c_dir = root.join(".cpp2rust/feat/c");
        std::fs::create_dir_all(c_dir.join("shim")).unwrap();

        // 原始实现 .cpp 必须真实存在才能 canonicalize。
        let shim_dir = root.join("shim");
        std::fs::create_dir_all(&shim_dir).unwrap();
        let impl_cpp = shim_dir.join("a_ffi.cpp");
        std::fs::write(&impl_cpp, "// impl\n").unwrap();

        // include 目录也需真实存在。
        let inc_dir = root.join("include");
        std::fs::create_dir_all(&inc_dir).unwrap();

        let cpp2rust = c_dir.join("shim/a_ffi.cpp.cpp2rust");
        std::fs::write(&cpp2rust, "// pp\n").unwrap();
        let opts = c_dir.join("shim/a_ffi.cpp.cpp2rust.opts");
        std::fs::write(
            &opts,
            format!("\"-std=c++11\" \"-I{}\" ", inc_dir.display()),
        )
        .unwrap();

        let meta = BuildMeta::collect(&[cpp2rust], &c_dir, root);
        assert_eq!(meta.cpp_std, Some("c++11".to_string()));
        assert_eq!(meta.include_dirs.len(), 1);
        assert!(meta.include_dirs[0].ends_with("include"));
        assert_eq!(meta.impl_sources.len(), 1);
        assert!(meta.impl_sources[0].ends_with("a_ffi.cpp"));
        assert!(!meta.is_empty());
    }

    #[test]
    fn collect_missing_opts_is_skipped() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let c_dir = root.join(".cpp2rust/feat/c");
        std::fs::create_dir_all(&c_dir).unwrap();
        let cpp2rust = c_dir.join("foo.cpp.cpp2rust");
        std::fs::write(&cpp2rust, "// pp\n").unwrap();
        // 不写 .opts，也不写原始 .cpp → 全部跳过，结果为空。
        let meta = BuildMeta::collect(&[cpp2rust], &c_dir, root);
        assert!(meta.is_empty());
    }
}
