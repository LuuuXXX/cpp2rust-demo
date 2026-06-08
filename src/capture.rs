use crate::error::Result;
use anyhow::anyhow;
#[cfg(any(target_os = "macos", windows))]
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// 将 hook 源文件内容直接嵌入 binary，确保 `cargo install` 后无需额外文件。
//
// Linux：使用 LD_PRELOAD + libhook.so，需要 hook.cpp 和 Makefile。
// macOS：SIP 会剥离 DYLD_INSERT_LIBRARIES，故改用与 Windows 相同的 shim 方式。
// Windows：使用 hook_shim.exe，PATH 注入拦截编译器。
#[cfg(target_os = "linux")]
const HOOK_CPP: &str = include_str!("../hook/hook.cpp");
#[cfg(target_os = "linux")]
const HOOK_MAKEFILE: &str = include_str!("../hook/Makefile");
#[cfg(any(target_os = "macos", windows))]
const HOOK_SHIM_RS: &str = include_str!("../hook/hook_shim.rs");

/// 构建平台对应的 hook 产物：
/// - Linux：从 `hook/Makefile` 构建 `libhook.so`
/// - macOS：用 `rustc` 编译 `hook_shim`（规避 SIP 限制）
/// - Windows：用 `rustc` 编译 `hook_shim.exe`
///
/// 若产物已是最新则跳过重新构建（快速路径）。返回产物路径。
pub fn build_hook() -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    return build_hook_linux();
    #[cfg(target_os = "macos")]
    return build_hook_macos();
    #[cfg(windows)]
    return build_hook_windows();
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    return Err(anyhow!(
        "capture hook is not supported on this platform (only Linux, macOS and Windows)"
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
    #[cfg(target_os = "linux")]
    return run_with_hook_linux(build_dir, cmd, project_root, feature_root, hook_artifact);
    #[cfg(target_os = "macos")]
    return run_with_hook_macos(build_dir, cmd, project_root, feature_root, hook_artifact);
    #[cfg(windows)]
    return run_with_hook_windows(build_dir, cmd, project_root, feature_root, hook_artifact);
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    return Err(anyhow!(
        "capture hook is not supported on this platform (only Linux, macOS and Windows)"
    ));
}

// ─────────────────────────────────────────────────────────────────
//  Linux 实现（LD_PRELOAD + libhook.so）
// ─────────────────────────────────────────────────────────────────

/// 从 `hook/Makefile` 构建 `libhook.so`。
///
/// 若 `libhook.so` 已存在且比 `hook.cpp` 更新，则跳过编译（快速路径）。
/// 返回 `libhook.so` 的路径。
#[cfg(target_os = "linux")]
fn build_hook_linux() -> Result<PathBuf> {
    let hook_dir = hook_dir()?;
    let so = hook_dir.join("libhook.so");
    let cpp = hook_dir.join("hook.cpp");

    // 快速路径：若 .so 比 hook.cpp 更新，则跳过重新编译。
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
        .map_err(|e| anyhow!("failed to run make in {}: {}", hook_dir.display(), e))?;

    if !status.success() {
        return Err(anyhow!("make failed in {}", hook_dir.display()));
    }

    if !so.exists() {
        return Err(anyhow!(
            "libhook.so not found after build at {}",
            so.display()
        ));
    }

    println!("Hook library built: {}", so.display());
    Ok(so)
}

/// 使用 LD_PRELOAD 设置为 libhook.so，执行用户提供的构建命令。
#[cfg(target_os = "linux")]
fn run_with_hook_linux(
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
        .map_err(|e| anyhow!("canonicalize {}: {}", project_root.display(), e))?;
    let abs_feature_root = feature_root
        .canonicalize()
        .map_err(|e| anyhow!("canonicalize {}: {}", feature_root.display(), e))?;
    let abs_hook = hook_so
        .canonicalize()
        .map_err(|e| anyhow!("canonicalize {}: {}", hook_so.display(), e))?;

    println!("Running build command: {}", cmd.join(" "));
    println!("  CPP2RUST_PROJECT_ROOT = {}", abs_project_root.display());
    println!("  CPP2RUST_FEATURE_ROOT = {}", abs_feature_root.display());
    println!("  LD_PRELOAD            = {}", abs_hook.display());
    println!();

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
            "build command failed with exit code {}",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

/// 从运行中二进制文件所在目录开始向上查找 `hook/` 目录（Linux）。
/// 若找不到，则将嵌入二进制的 hook 源文件解压到用户数据目录，
/// 以便 `cargo install` 用户无需单独检出代码即可使用。
#[cfg(target_os = "linux")]
fn hook_dir() -> Result<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("hook");
            if candidate.join("Makefile").exists() {
                return Ok(candidate);
            }
            if let Some(workspace) = parent
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
            {
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

    // 最终回退：将嵌入的源文件解压到用户数据目录。
    ensure_hook_data_dir()
}

/// 返回用户 hook 数据目录，不存在或文件过时时创建目录并写入嵌入的 `hook.cpp` / `Makefile`（Linux）。
///
/// 目录路径：`$XDG_DATA_HOME/cpp2rust-demo/hook/`（默认 `~/.local/share/cpp2rust-demo/hook/`）
#[cfg(target_os = "linux")]
fn ensure_hook_data_dir() -> Result<PathBuf> {
    let base = data_dir().ok_or_else(|| anyhow!("cannot determine user data directory"))?;
    let hook_dir = base.join("cpp2rust-demo").join("hook");

    std::fs::create_dir_all(&hook_dir)
        .map_err(|e| anyhow!("create_dir_all {}: {}", hook_dir.display(), e))?;

    // 若 hook.cpp 不存在或内容有变化，则写入（二进制更新时自动升级）。
    write_if_changed(&hook_dir.join("hook.cpp"), HOOK_CPP)?;
    // 若 Makefile 不存在或内容有变化，则写入。
    write_if_changed(&hook_dir.join("Makefile"), HOOK_MAKEFILE)?;

    Ok(hook_dir)
}

/// 若 `path` 不存在或内容与 `content` 不同，则写入文件（Linux）。
///
/// 避免不必要的写入，以免触发文件系统 mtime 变更（影响 `libhook.so` 的快速路径判断）。
#[cfg(target_os = "linux")]
fn write_if_changed(path: &Path, content: &str) -> Result<()> {
    let needs_write = match std::fs::read_to_string(path) {
        Ok(existing) => existing != content,
        Err(_) => true,
    };
    if needs_write {
        std::fs::write(path, content).map_err(|e| anyhow!("write {}: {}", path.display(), e))?;
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────
//  macOS 实现（PATH 注入 + hook_shim）
//
//  macOS SIP（System Integrity Protection）会在 exec 系统调用时自动剥离
//  DYLD_INSERT_LIBRARIES 等 DYLD_* 环境变量（针对启用了 hardened runtime 的
//  系统 shell，如 /bin/bash、/bin/zsh）。因此无法可靠地使用 DYLD_INSERT_LIBRARIES
//  拦截编译器，转而采用与 Windows 相同的 shim 方式：将 hook_shim 以编译器真名
//  复制到临时目录并置于 PATH 最前面。
// ─────────────────────────────────────────────────────────────────

/// 将 `hook_shim.rs` 写入数据目录并使用 `rustc` 编译为 `hook_shim`（macOS）。
///
/// 产物路径：`~/Library/Application Support/cpp2rust-demo/hook/hook_shim`
/// 若 shim 源码无变化则复用已有产物（快速路径）。
#[cfg(target_os = "macos")]
fn build_hook_macos() -> Result<PathBuf> {
    let base = data_dir().ok_or_else(|| anyhow!("cannot determine user data directory"))?;
    let hook_dir = base.join("cpp2rust-demo").join("hook");
    std::fs::create_dir_all(&hook_dir)
        .map_err(|e| anyhow!("create_dir_all {}: {}", hook_dir.display(), e))?;

    let shim_rs = hook_dir.join("hook_shim.rs");
    let shim_bin = hook_dir.join("hook_shim");

    let needs_write = match std::fs::read_to_string(&shim_rs) {
        Ok(existing) => existing != HOOK_SHIM_RS,
        Err(_) => true,
    };
    if needs_write {
        std::fs::write(&shim_rs, HOOK_SHIM_RS)
            .map_err(|e| anyhow!("write {}: {}", shim_rs.display(), e))?;
    }

    if shim_bin.exists() && !needs_write {
        println!("Hook shim up-to-date: {}", shim_bin.display());
        return Ok(shim_bin);
    }

    println!("Compiling hook shim from {}...", shim_rs.display());
    let status = Command::new("rustc")
        .args(["--edition", "2021", "-o"])
        .arg(&shim_bin)
        .arg(&shim_rs)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| anyhow!("failed to run rustc (is Rust installed?): {}", e))?;

    if !status.success() {
        return Err(anyhow!("rustc failed to compile hook_shim.rs"));
    }
    if !shim_bin.exists() {
        return Err(anyhow!(
            "hook_shim not found after build at {}",
            shim_bin.display()
        ));
    }

    // 确保 shim 具有执行权限
    set_executable(&shim_bin)?;
    println!("Hook shim built: {}", shim_bin.display());
    Ok(shim_bin)
}

/// 通过 PATH 注入将 `hook_shim` 伪装成真实编译器，然后执行用户构建命令（macOS）。
///
/// 流程：
///  1. 在临时目录中将 hook_shim 以真实编译器基名（如 `clang++` / `g++`）命名。
///  2. 将该临时目录插入 PATH 最前面。
///  3. 设置 `CPP2RUST_REAL_CC`、`CPP2RUST_COMPILER_KIND=gnu`、
///     `CPP2RUST_PROJECT_ROOT`、`CPP2RUST_FEATURE_ROOT`。
///  4. 执行构建命令；shim 拦截编译调用并生成 .cpp2rust 文件。
#[cfg(target_os = "macos")]
fn run_with_hook_macos(
    build_dir: &Path,
    cmd: &[String],
    project_root: &Path,
    feature_root: &Path,
    hook_bin: &Path,
) -> Result<()> {
    if cmd.is_empty() {
        return Err(anyhow!("build command is empty"));
    }

    let abs_project_root = project_root
        .canonicalize()
        .map_err(|e| anyhow!("canonicalize {}: {}", project_root.display(), e))?;
    let abs_feature_root = feature_root
        .canonicalize()
        .map_err(|e| anyhow!("canonicalize {}: {}", feature_root.display(), e))?;

    let real_cc = detect_macos_cxx_compiler()
        .ok_or_else(|| anyhow!("no C++ compiler (clang++/g++/c++) found in PATH"))?;

    let tmp_dir = tempfile::Builder::new()
        .prefix("cpp2rust-shim-")
        .tempdir()
        .map_err(|e| anyhow!("tempdir: {}", e))?;

    // 为 macOS 上三个主要的 C++ 编译器别名创建 shim 副本：
    // 构建脚本可能硬编码 g++、clang++ 或 c++，统一拦截以确保 capture 正常工作。
    let shim_names = ["clang++", "g++", "c++"];
    for name in &shim_names {
        let alias = tmp_dir.path().join(name);
        std::fs::copy(hook_bin, &alias)
            .map_err(|e| anyhow!("copy shim → {}: {}", alias.display(), e))?;
        set_executable(&alias)?;
    }
    // 若检测到的真实编译器 basename 不在上述三个名字中（如 g++-14 等），
    // 也为其创建一份 shim（尽力而为，覆盖非标准命名场景）。
    let cc_basename = real_cc
        .file_name()
        .ok_or_else(|| anyhow!("real_cc has no filename"))?;
    if !shim_names.iter().any(|n| OsStr::new(n) == cc_basename) {
        let shim_alias = tmp_dir.path().join(cc_basename);
        std::fs::copy(hook_bin, &shim_alias)
            .map_err(|e| anyhow!("copy shim → {}: {}", shim_alias.display(), e))?;
        set_executable(&shim_alias)?;
    }

    let old_path = std::env::var_os("PATH").unwrap_or_default();
    let new_path = std::env::join_paths(
        std::iter::once(tmp_dir.path().to_path_buf()).chain(std::env::split_paths(&old_path)),
    )
    .map_err(|e| anyhow!("join_paths: {}", e))?;

    println!("Running build command: {}", cmd.join(" "));
    println!("  CPP2RUST_PROJECT_ROOT  = {}", abs_project_root.display());
    println!("  CPP2RUST_FEATURE_ROOT  = {}", abs_feature_root.display());
    println!("  CPP2RUST_REAL_CC       = {}", real_cc.display());
    println!("  CPP2RUST_COMPILER_KIND = gnu");
    println!();

    let status = Command::new(&cmd[0])
        .args(&cmd[1..])
        .current_dir(build_dir)
        .env("PATH", new_path)
        .env("CPP2RUST_REAL_CC", &real_cc)
        .env("CPP2RUST_COMPILER_KIND", "gnu")
        .env("CPP2RUST_PROJECT_ROOT", &abs_project_root)
        .env("CPP2RUST_FEATURE_ROOT", &abs_feature_root)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| anyhow!("failed to spawn '{}': {}", cmd[0], e))?;

    if !status.success() {
        return Err(anyhow!(
            "build command failed with exit code {}",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

/// 为 hook_shim 副本设置可执行权限位（macOS）。
#[cfg(target_os = "macos")]
fn set_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)
        .map_err(|e| anyhow!("metadata {}: {}", path.display(), e))?
        .permissions();
    perms.set_mode(perms.mode() | 0o111);
    std::fs::set_permissions(path, perms)
        .map_err(|e| anyhow!("set_permissions {}: {}", path.display(), e))?;
    Ok(())
}

/// 在当前 PATH 中搜索 C++ 编译器（macOS）。
///
/// 搜索顺序：`clang++` → `g++` → `c++`。
/// - `clang++`：优先使用 Homebrew LLVM（已由调用方置于 PATH 最前）；
/// - `g++`：备选 Homebrew GCC；
/// - `c++`：最终后备（macOS 上通常是 Xcode/CommandLineTools 提供的 Apple Clang 别名）。
/// 首个在 PATH 目录中找到的可执行文件即为返回值。
#[cfg(target_os = "macos")]
fn detect_macos_cxx_compiler() -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let candidates = ["clang++", "g++", "c++"];
    for dir in std::env::split_paths(&path_var) {
        for &name in &candidates {
            let full = dir.join(name);
            if full.is_file() {
                return Some(full);
            }
        }
    }
    None
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
    std::fs::create_dir_all(&hook_dir)
        .map_err(|e| anyhow!("create_dir_all {}: {}", hook_dir.display(), e))?;

    let shim_rs = hook_dir.join("hook_shim.rs");
    let shim_exe = hook_dir.join("hook_shim.exe");

    // 若源码有变化则写入（触发重新编译）
    let needs_write = match std::fs::read_to_string(&shim_rs) {
        Ok(existing) => existing != HOOK_SHIM_RS,
        Err(_) => true,
    };
    if needs_write {
        std::fs::write(&shim_rs, HOOK_SHIM_RS)
            .map_err(|e| anyhow!("write {}: {}", shim_rs.display(), e))?;
    }

    // 快速路径：若 .exe 比 .rs 更新则跳过编译
    if shim_exe.exists() && !needs_write {
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
        .map_err(|e| anyhow!("failed to run rustc (is Rust installed?): {}", e))?;

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
        .map_err(|e| anyhow!("canonicalize {}: {}", project_root.display(), e))?;
    let abs_feature_root = canonicalize_no_verbatim(feature_root)
        .map_err(|e| anyhow!("canonicalize {}: {}", feature_root.display(), e))?;

    // 找到真实 C++ 编译器（绕过 shim 目录本身），同时获取编译器类型
    let (real_cc, cc_kind) = detect_windows_cxx_compiler()
        .ok_or_else(|| anyhow!("no C++ compiler (g++/clang++/cl.exe) found in PATH"))?;

    // 将 shim 放入临时目录，以真实编译器基名命名
    let tmp_dir = tempfile::Builder::new()
        .prefix("cpp2rust-shim-")
        .tempdir()
        .map_err(|e| anyhow!("tempdir: {}", e))?;
    let cc_basename = real_cc
        .file_name()
        .ok_or_else(|| anyhow!("real_cc has no filename"))?;
    let shim_alias = tmp_dir.path().join(cc_basename);
    std::fs::copy(hook_exe, &shim_alias)
        .map_err(|e| anyhow!("copy shim → {}: {}", shim_alias.display(), e))?;

    // 将临时目录插入 PATH 最前面
    let old_path = std::env::var_os("PATH").unwrap_or_default();
    let new_path = std::env::join_paths(
        std::iter::once(tmp_dir.path().to_path_buf()).chain(std::env::split_paths(&old_path)),
    )
    .map_err(|e| anyhow!("join_paths: {}", e))?;

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
        .map_err(|e| anyhow!("failed to spawn '{}': {}", cmd[0], e))?;

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
