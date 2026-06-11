use crate::error::{Cpp2RustError, Result};
use anyhow::{anyhow, Context as _};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// 将 hook 源文件内容直接嵌入 binary，确保 `cargo install` 后无需额外文件。
#[cfg(unix)]
const HOOK_CPP: &str = include_str!("../hook/hook.cpp");
#[cfg(unix)]
const HOOK_MAKEFILE: &str = include_str!("../hook/Makefile");
#[cfg(windows)]
const HOOK_SHIM_RS: &str = include_str!("../hook/hook_shim.rs");

/// 从 `hook/Makefile` 构建 `libhook.so`（Unix），或编译 `hook_shim.exe`（Windows）。
///
/// 若产物已是最新则跳过重新构建（快速路径）。返回产物路径。
pub fn build_hook() -> Result<PathBuf> {
    #[cfg(unix)]
    return build_hook_unix();
    #[cfg(windows)]
    return build_hook_windows();
    #[cfg(not(any(unix, windows)))]
    return Err(anyhow!(
        "capture hook is not supported on this platform (only Unix and Windows)"
    ));
}

/// 使用平台对应的 hook 机制运行用户提供的构建命令，并捕获 .cpp2rust 文件。
pub fn run_with_hook(
    build_dir: &Path,
    cmd: &[String],
    project_root: &Path,
    feature_root: &Path,
    hook_artifact: &Path,
) -> Result<()> {
    #[cfg(unix)]
    return run_with_hook_unix(build_dir, cmd, project_root, feature_root, hook_artifact);
    #[cfg(windows)]
    return run_with_hook_windows(build_dir, cmd, project_root, feature_root, hook_artifact);
    #[cfg(not(any(unix, windows)))]
    return Err(anyhow!(
        "capture hook is not supported on this platform (only Unix and Windows)"
    ));
}

// ─────────────────────────────────────────────────────────────────
//  Unix 实现（LD_PRELOAD + libhook.so）
// ─────────────────────────────────────────────────────────────────

/// 从 `hook/Makefile` 构建 `libhook.so`（Linux）或 `libhook.dylib`（macOS）。
///
/// 若产物已是最新则跳过重新构建（快速路径）。返回产物路径。
#[cfg(unix)]
fn build_hook_unix() -> Result<PathBuf> {
    let hook_dir = hook_dir()?;
    // macOS 产物为 .dylib；Linux 产物为 .so
    let lib_name = if cfg!(target_os = "macos") {
        "libhook.dylib"
    } else {
        "libhook.so"
    };
    let so = hook_dir.join(lib_name);
    let cpp = hook_dir.join("hook.cpp");

    // 快速路径：若 .so/.dylib 比 hook.cpp 更新，则跳过重新编译。
    // 注意：上方的 hook_dir() 已在最终回退路径中调用 ensure_hook_data_dir()，
    // 因此在此检查之前，hook.cpp 已保证存在于数据目录中。
    if so.exists() && cpp.exists() {
        if let (Ok(so_meta), Ok(cpp_meta)) = (so.metadata(), cpp.metadata()) {
            if let (Ok(so_mtime), Ok(cpp_mtime)) = (so_meta.modified(), cpp_meta.modified()) {
                if so_mtime >= cpp_mtime {
                    println!("Hook library up-to-date: {}", so.display());
                    return Ok(so);
                }
            }
        }
    }

    println!("Building hook library from {}...", hook_dir.display());
    let status = Command::new("make")
        .current_dir(&hook_dir)
        .arg("-s")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to run make in {}", hook_dir.display()))?;

    if !status.success() {
        return Err(anyhow!("make failed in {}", hook_dir.display()));
    }

    if !so.exists() {
        return Err(anyhow!(
            "{} not found after build at {}",
            lib_name,
            so.display()
        ));
    }

    println!("Hook library built: {}", so.display());
    Ok(so)
}

/// 使用 DYLD_INSERT_LIBRARIES（macOS）或 LD_PRELOAD（Linux）注入 hook 库，执行用户提供的构建命令。
#[cfg(unix)]
fn run_with_hook_unix(
    build_dir: &Path,
    cmd: &[String],
    project_root: &Path,
    feature_root: &Path,
    hook_so: &Path,
) -> Result<()> {
    if cmd.is_empty() {
        return Err(anyhow!("build command is empty"));
    }

    let abs_project_root = project_root
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.display()))?;
    let abs_feature_root = feature_root
        .canonicalize()
        .with_context(|| format!("canonicalize {}", feature_root.display()))?;
    let abs_hook = hook_so
        .canonicalize()
        .with_context(|| format!("canonicalize {}", hook_so.display()))?;

    // macOS 使用 DYLD_INSERT_LIBRARIES；Linux 使用 LD_PRELOAD
    #[cfg(target_os = "macos")]
    let inject_var = "DYLD_INSERT_LIBRARIES";
    #[cfg(not(target_os = "macos"))]
    let inject_var = "LD_PRELOAD";

    println!("Running build command: {}", cmd.join(" "));
    println!("  CPP2RUST_PROJECT_ROOT = {}", abs_project_root.display());
    println!("  CPP2RUST_FEATURE_ROOT = {}", abs_feature_root.display());
    println!("  {}            = {}", inject_var, abs_hook.display());
    println!();

    let status = Command::new(&cmd[0])
        .args(&cmd[1..])
        .current_dir(build_dir)
        .env(inject_var, &abs_hook)
        .env("CPP2RUST_PROJECT_ROOT", &abs_project_root)
        .env("CPP2RUST_FEATURE_ROOT", &abs_feature_root)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to spawn '{}'", cmd[0]))?;

    if !status.success() {
        return Err(anyhow!(
            "build command failed with exit code {}",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

/// 从运行中二进制文件所在目录开始向上查找 `hook/` 目录。
/// 若找不到，则将嵌入二进制的 hook 源文件解压到用户数据目录，
/// 以便 `cargo install` 用户无需单独检出代码即可使用。
#[cfg(unix)]
fn hook_dir() -> Result<PathBuf> {
    // 从 exe 所在目录与当前工作目录分别向上搜索，最多 5 层。
    let search_roots: Vec<PathBuf> = [
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf())),
        std::env::current_dir().ok(),
    ]
    .into_iter()
    .filter_map(|opt| opt)
    .collect();

    const MAX_DEPTH: usize = 5;
    for start in search_roots {
        let mut dir: &std::path::Path = &start;
        for _ in 0..=MAX_DEPTH {
            let candidate = dir.join("hook");
            if candidate.join("Makefile").exists() {
                return Ok(candidate);
            }
            match dir.parent() {
                Some(p) => dir = p,
                None => break,
            }
        }
    }

    // 最终回退：将嵌入的源文件解压到用户数据目录。
    ensure_hook_data_dir()
}

/// 返回用户 hook 数据目录，不存在或文件过时时创建目录并写入嵌入的 `hook.cpp` / `Makefile`。
///
/// 目录路径：
/// - Linux / 其他：`$XDG_DATA_HOME/cpp2rust-demo/hook/`
///   （默认 `~/.local/share/cpp2rust-demo/hook/`）
/// - macOS：`~/Library/Application Support/cpp2rust-demo/hook/`
#[cfg(unix)]
fn ensure_hook_data_dir() -> Result<PathBuf> {
    let base = data_dir().ok_or_else(|| anyhow!("cannot determine user data directory"))?;
    let hook_dir = base.join("cpp2rust-demo").join("hook");

    std::fs::create_dir_all(&hook_dir).map_err(|e| {
        Cpp2RustError::IoError(format!("create_dir_all {}: {e}", hook_dir.display()))
    })?;

    // 若 hook.cpp 不存在或内容有变化，则写入（二进制更新时自动升级）。
    let _ = write_if_changed(&hook_dir.join("hook.cpp"), HOOK_CPP)?;
    // 若 Makefile 不存在或内容有变化，则写入。
    let _ = write_if_changed(&hook_dir.join("Makefile"), HOOK_MAKEFILE)?;

    Ok(hook_dir)
}

/// 若 `path` 不存在或内容与 `content` 不同，则写入文件；返回是否真正写入。
///
/// 避免不必要的写入，以免触发文件系统 mtime 变更（影响 `libhook.so` 的快速路径判断）。
/// Unix 和 Windows 均可使用。返回 `true` 表示文件已被写入，`false` 表示内容相同未写入。
fn write_if_changed(path: &Path, content: &str) -> Result<bool> {
    let needs_write = match std::fs::read_to_string(path) {
        Ok(existing) => existing != content,
        Err(_) => true,
    };
    if needs_write {
        std::fs::write(path, content)
            .map_err(|e| Cpp2RustError::IoError(format!("write {}: {e}", path.display())))?;
    }
    Ok(needs_write)
}

// ─────────────────────────────────────────────────────────────────
//  Windows 实现（PATH 注入 + hook_shim.exe）
//  支持 GNU（g++/clang++）和 MSVC（cl.exe）两种编译器
// ─────────────────────────────────────────────────────────────────

/// 将 `hook_shim.rs` 写入数据目录并使用 `rustc` 编译为 `hook_shim.exe`。
///
/// 产物路径：`%APPDATA%\cpp2rust-demo\hook\hook_shim.exe`
/// 若 shim 源码无变化则复用已有产物（快速路径）。
///
/// 同一个 `hook_shim.exe` 同时支持 GNU 和 MSVC 两种模式：
/// 运行时通过 `CPP2RUST_COMPILER_KIND` 环境变量（由 `run_with_hook_windows` 设置）区分。
#[cfg(windows)]
fn build_hook_windows() -> Result<PathBuf> {
    let base = data_dir().ok_or_else(|| anyhow!("cannot determine user data directory"))?;
    let hook_dir = base.join("cpp2rust-demo").join("hook");
    std::fs::create_dir_all(&hook_dir).map_err(|e| {
        Cpp2RustError::IoError(format!("create_dir_all {}: {e}", hook_dir.display()))
    })?;

    let shim_rs = hook_dir.join("hook_shim.rs");
    let shim_exe = hook_dir.join("hook_shim.exe");

    // 若源码有变化则写入（触发重新编译）；was_written = true 表示 shim 源码已更新。
    let was_written = write_if_changed(&shim_rs, HOOK_SHIM_RS)?;

    // 快速路径：若 .exe 存在且 shim 源码无变化则跳过编译
    if shim_exe.exists() && !was_written {
        println!("Hook shim up-to-date: {}", shim_exe.display());
        return Ok(shim_exe);
    }

    println!("Compiling hook shim from {}...", shim_rs.display());
    let status = Command::new("rustc")
        .args(["--edition", "2021", "-o"])
        .arg(&shim_exe)
        .arg(&shim_rs)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to run rustc (is Rust installed?)")?;

    if !status.success() {
        return Err(anyhow!("rustc failed to compile hook_shim.rs"));
    }
    if !shim_exe.exists() {
        return Err(anyhow!(
            "hook_shim.exe not found after build at {}",
            shim_exe.display()
        ));
    }

    println!("Hook shim built: {}", shim_exe.display());
    Ok(shim_exe)
}

/// Windows 编译器类型，用于向 hook_shim 传递正确的预处理策略。
#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowsCompilerKind {
    /// GCC/Clang（g++.exe / clang++.exe / c++.exe）
    Gnu,
    /// MSVC（cl.exe）
    Msvc,
}

#[cfg(windows)]
impl WindowsCompilerKind {
    /// 对应 `CPP2RUST_COMPILER_KIND` 环境变量的字符串值。
    fn env_value(self) -> &'static str {
        match self {
            WindowsCompilerKind::Gnu => "gnu",
            WindowsCompilerKind::Msvc => "msvc",
        }
    }
}

/// 通过 PATH 注入将 `hook_shim.exe` 伪装成真实编译器，然后执行用户构建命令。
///
/// 流程：
///  1. 在临时目录中将 hook_shim.exe 以真实编译器基名（如 `g++.exe` / `cl.exe`）命名。
///  2. 将该临时目录插入 PATH 最前面。
///  3. 设置 `CPP2RUST_REAL_CC`、`CPP2RUST_COMPILER_KIND`、`CPP2RUST_PROJECT_ROOT`、
///     `CPP2RUST_FEATURE_ROOT`。
///  4. 执行构建命令；shim 拦截编译调用并生成 .cpp2rust 文件。
#[cfg(windows)]
fn run_with_hook_windows(
    build_dir: &Path,
    cmd: &[String],
    project_root: &Path,
    feature_root: &Path,
    hook_exe: &Path,
) -> Result<()> {
    if cmd.is_empty() {
        return Err(anyhow!("build command is empty"));
    }

    let abs_project_root = canonicalize_no_verbatim(project_root)
        .with_context(|| format!("canonicalize {}", project_root.display()))?;
    let abs_feature_root = canonicalize_no_verbatim(feature_root)
        .with_context(|| format!("canonicalize {}", feature_root.display()))?;

    // 找到真实 C++ 编译器（绕过 shim 目录本身），同时获取编译器类型
    let (real_cc, cc_kind) = detect_windows_cxx_compiler()
        .ok_or_else(|| anyhow!("no C++ compiler (g++/clang++/cl.exe) found in PATH"))?;

    // 将 shim 放入临时目录，以真实编译器基名命名
    let tmp_dir = tempfile::Builder::new()
        .prefix("cpp2rust-shim-")
        .tempdir()
        .context("tempdir")?;
    let cc_basename = real_cc
        .file_name()
        .ok_or_else(|| anyhow!("real_cc has no filename"))?;
    let shim_alias = tmp_dir.path().join(cc_basename);
    std::fs::copy(hook_exe, &shim_alias)
        .with_context(|| format!("copy shim → {}", shim_alias.display()))?;

    // 将临时目录插入 PATH 最前面
    let old_path = std::env::var_os("PATH").unwrap_or_default();
    let new_path = std::env::join_paths(
        std::iter::once(tmp_dir.path().to_path_buf()).chain(std::env::split_paths(&old_path)),
    )
    .context("join_paths")?;

    println!("Running build command: {}", cmd.join(" "));
    println!("  CPP2RUST_PROJECT_ROOT  = {}", abs_project_root.display());
    println!("  CPP2RUST_FEATURE_ROOT  = {}", abs_feature_root.display());
    println!("  CPP2RUST_REAL_CC       = {}", real_cc.display());
    println!("  CPP2RUST_COMPILER_KIND = {}", cc_kind.env_value());
    println!();

    let status = Command::new(&cmd[0])
        .args(&cmd[1..])
        .current_dir(build_dir)
        .env("PATH", new_path)
        .env("CPP2RUST_REAL_CC", &real_cc)
        .env("CPP2RUST_COMPILER_KIND", cc_kind.env_value())
        .env("CPP2RUST_PROJECT_ROOT", &abs_project_root)
        .env("CPP2RUST_FEATURE_ROOT", &abs_feature_root)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to spawn '{}'", cmd[0]))?;

    if !status.success() {
        return Err(anyhow!(
            "build command failed with exit code {}",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

/// Windows canonicalize() 会产生 `\\?\` 前缀（扩展路径格式）。
/// 该前缀会导致后续 strip_prefix 匹配失败，因此去除它，返回普通 Windows 绝对路径。
///
/// 在 Windows 上会去掉 `\\?\` 前缀；其他平台直接调用标准 canonicalize。
#[cfg(windows)]
fn canonicalize_no_verbatim(p: &std::path::Path) -> std::io::Result<PathBuf> {
    let canonical = p.canonicalize()?;
    // 去掉 \\?\ 前缀（Windows 扩展路径）
    let s = canonical.to_string_lossy();
    if let Some(stripped) = s.strip_prefix("\\\\?\\") {
        Ok(PathBuf::from(stripped))
    } else {
        Ok(canonical)
    }
}

/// 在当前 PATH 中搜索 C++ 编译器，返回第一个找到的完整路径及其类型。
///
/// 搜索顺序（GNU 优先，以便在同时安装 MinGW 和 MSVC 时保持向后兼容）：
/// 1. g++.exe / clang++.exe / c++.exe（GNU ABI）
/// 2. cl.exe（MSVC）
///
/// 每个 PATH 目录只遍历一次：先检查所有 GNU 候选，再检查 cl.exe。
/// 若系统同时存在 GNU 和 MSVC 编译器，GNU 将优先被选择。
/// 如需强制使用 MSVC，可在调用 `cpp2rust-demo init` 之前将 MSVC 的 bin 目录排列在 PATH 最前面
/// 并确保 g++/clang++ 不在 PATH 中。
#[cfg(windows)]
fn detect_windows_cxx_compiler() -> Option<(PathBuf, WindowsCompilerKind)> {
    let path_var = std::env::var_os("PATH")?;

    let gnu_candidates = ["g++.exe", "clang++.exe", "c++.exe"];
    let mut msvc_candidate: Option<PathBuf> = None;

    // 单次遍历 PATH，同时检查 GNU 和 MSVC 候选
    for dir in std::env::split_paths(&path_var) {
        // 优先检查 GNU 编译器
        for name in &gnu_candidates {
            let full = dir.join(name);
            if full.is_file() {
                return Some((full, WindowsCompilerKind::Gnu));
            }
        }
        // 记录第一个找到的 cl.exe（若尚未找到 GNU，则作为后备）
        if msvc_candidate.is_none() {
            let cl = dir.join("cl.exe");
            if cl.is_file() {
                msvc_candidate = Some(cl);
            }
        }
    }

    // 只有在没有找到任何 GNU 编译器时才使用 cl.exe
    msvc_candidate.map(|p| (p, WindowsCompilerKind::Msvc))
}

// ─────────────────────────────────────────────────────────────────
//  共用：平台相关的数据目录 + home 目录
// ─────────────────────────────────────────────────────────────────

/// 平台相关的基础数据目录（不含应用子路径）。
fn data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs_home().map(|h| h.join("Library").join("Application Support"))
    }
    #[cfg(windows)]
    {
        // 优先使用 %APPDATA%（通常为 C:\Users\<user>\AppData\Roaming）
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        // 优先使用 XDG_DATA_HOME；回退到 ~/.local/share。
        if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
            let p = PathBuf::from(xdg);
            if p.is_absolute() {
                return Some(p);
            }
        }
        dirs_home().map(|h| h.join(".local").join("share"))
    }
}

/// 返回当前用户的 home 目录。
///
/// - POSIX（Linux / macOS）：使用 `HOME` 环境变量。
/// - Windows：依次尝试 `USERPROFILE`、`HOMEDRIVE` + `HOMEPATH`。
fn dirs_home() -> Option<PathBuf> {
    // POSIX: HOME
    if let Some(h) = std::env::var_os("HOME") {
        let p = PathBuf::from(h);
        if p.is_absolute() {
            return Some(p);
        }
    }
    // Windows: USERPROFILE
    #[cfg(windows)]
    {
        if let Some(h) = std::env::var_os("USERPROFILE") {
            let p = PathBuf::from(h);
            if p.is_absolute() {
                return Some(p);
            }
        }
        if let (Some(drive), Some(path)) =
            (std::env::var_os("HOMEDRIVE"), std::env::var_os("HOMEPATH"))
        {
            let mut full = PathBuf::from(drive);
            full.push(path);
            if full.is_absolute() {
                return Some(full);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── write_if_changed ────────────────────────────────────────────────────────

    #[test]
    fn write_if_changed_creates_file_when_not_exists() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("file.txt");
        let written = write_if_changed(&path, "hello").unwrap();
        assert!(written, "文件不存在时应返回 true（已写入）");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello");
    }

    #[test]
    fn write_if_changed_returns_false_when_content_same() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("file.txt");
        std::fs::write(&path, "same").unwrap();
        let mtime_before = std::fs::metadata(&path).unwrap().modified().unwrap();

        let written = write_if_changed(&path, "same").unwrap();
        assert!(!written, "内容相同时应返回 false（未写入）");

        let mtime_after = std::fs::metadata(&path).unwrap().modified().unwrap();
        assert_eq!(mtime_before, mtime_after, "内容相同时不应修改 mtime");
    }

    #[test]
    fn write_if_changed_overwrites_when_content_differs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("file.txt");
        std::fs::write(&path, "old").unwrap();

        let written = write_if_changed(&path, "new").unwrap();
        assert!(written, "内容不同时应返回 true（已写入）");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
    }

    // ── dirs_home ───────────────────────────────────────────────────────────────

    #[test]
    fn dirs_home_returns_some_on_standard_env() {
        // 在标准 CI 环境（HOME 已设置）下应返回 Some
        if std::env::var_os("HOME").is_some() || std::env::var_os("USERPROFILE").is_some() {
            assert!(dirs_home().is_some(), "标准环境下 dirs_home() 应返回 Some");
        }
    }
}
