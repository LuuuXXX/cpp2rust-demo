use crate::error::Result;
use anyhow::anyhow;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// ── Unix：将 hook 源文件内容直接嵌入 binary，确保 `cargo install` 后无需额外文件 ──
#[cfg(unix)]
const HOOK_CPP: &str = include_str!("../hook/hook.cpp");
#[cfg(unix)]
const HOOK_MAKEFILE: &str = include_str!("../hook/Makefile");

// ── Windows：将预构建的 hook-wrapper.exe 字节嵌入 binary ──
// build.rs 在 Windows 目标时构建 hook-wrapper 并通过 rustc-env 传递路径。
#[cfg(windows)]
const HOOK_WRAPPER_EXE: &[u8] = include_bytes!(env!("CPP2RUST_HOOK_WRAPPER_EXE"));

/// 准备钩子产物：
///   - Unix：编译 libhook.so，返回 .so 路径
///   - Windows：解压 hook-wrapper.exe，返回 .exe 路径
pub fn build_hook() -> Result<PathBuf> {
    #[cfg(unix)]
    return build_hook_unix();
    #[cfg(windows)]
    return build_hook_windows();
    #[cfg(not(any(unix, windows)))]
    return Err(anyhow!("unsupported platform"));
}

/// 使用钩子执行构建命令：
///   - Unix：通过 LD_PRELOAD 注入 libhook.so
///   - Windows：通过 PATH 注入 hook-wrapper.exe 副本
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
    return Err(anyhow!("unsupported platform"));
}

// ══════════════════════════════════════════════════════════════════════════
//  Unix 实现
// ══════════════════════════════════════════════════════════════════════════

/// 从 `hook/Makefile` 构建 `libhook.so`。
///
/// 若 `libhook.so` 已存在且比 `hook.cpp` 更新，则跳过编译（快速路径）。
/// 返回 `libhook.so` 的路径。
#[cfg(unix)]
fn build_hook_unix() -> Result<PathBuf> {
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

// ══════════════════════════════════════════════════════════════════════════
//  Windows 实现
// ══════════════════════════════════════════════════════════════════════════

/// 将内嵌的 hook-wrapper.exe 解压到用户数据目录并返回其路径。
#[cfg(windows)]
fn build_hook_windows() -> Result<PathBuf> {
    ensure_hook_wrapper_exe()
}

/// 确保 hook-wrapper.exe 已解压到数据目录，若已是最新则跳过写入。
/// 返回 hook-wrapper.exe 的路径。
#[cfg(windows)]
fn ensure_hook_wrapper_exe() -> Result<PathBuf> {
    let base = data_dir().ok_or_else(|| anyhow!("cannot determine user data directory"))?;
    let hook_dir = base.join("cpp2rust-demo").join("hook");

    std::fs::create_dir_all(&hook_dir)
        .map_err(|e| anyhow!("create_dir_all {}: {}", hook_dir.display(), e))?;

    let exe_path = hook_dir.join("hook-wrapper.exe");

    // 若已存在且内容相同则跳过写入（按字节比对）
    let needs_write = match std::fs::read(&exe_path) {
        Ok(existing) => existing.as_slice() != HOOK_WRAPPER_EXE,
        Err(_) => true,
    };
    if needs_write {
        std::fs::write(&exe_path, HOOK_WRAPPER_EXE)
            .map_err(|e| anyhow!("write {}: {}", exe_path.display(), e))?;
        println!("Hook wrapper updated: {}", exe_path.display());
    } else {
        println!("Hook wrapper up-to-date: {}", exe_path.display());
    }

    Ok(exe_path)
}

/// 去掉 Windows `\\?\` 长路径前缀，得到可直接传给子进程的普通路径。
#[cfg(windows)]
fn strip_unc_prefix(p: &Path) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped.to_string())
    } else {
        p.to_path_buf()
    }
}

/// 规范化路径并去掉 `\\?\` 前缀。
#[cfg(windows)]
fn win_canonical(p: &Path) -> Result<PathBuf> {
    let c = p
        .canonicalize()
        .map_err(|e| anyhow!("canonicalize {}: {}", p.display(), e))?;
    Ok(strip_unc_prefix(&c))
}

/// Windows 版 run_with_hook：
///   1. 在临时目录中为每个目标编译器名创建 hook-wrapper.exe 的副本
///   2. 将临时目录前置到 PATH，通过子进程环境变量传递（不影响父进程）
///   3. 执行构建命令，结束后临时目录自动清理
#[cfg(windows)]
fn run_with_hook_windows(
    build_dir: &Path,
    cmd: &[String],
    project_root: &Path,
    feature_root: &Path,
    hook_wrapper_exe: &Path,
) -> Result<()> {
    if cmd.is_empty() {
        return Err(anyhow!("build command is empty"));
    }

    let abs_project_root = win_canonical(project_root)?;
    let abs_feature_root = win_canonical(feature_root)?;
    let abs_hook = win_canonical(hook_wrapper_exe)?;

    // 在系统临时目录中创建一个独立的 wrapper 工作目录
    let temp_dir = tempfile::tempdir()
        .map_err(|e| anyhow!("create tempdir for hook wrappers: {}", e))?;

    // 为每个目标编译器名创建 hook-wrapper.exe 的硬链接（失败则回退到拷贝）
    let compiler_names = [
        "cl.exe",
        "clang-cl.exe",
        "g++.exe",
        "clang++.exe",
        "c++.exe",
        "g++",
        "clang++",
        "c++",
    ];
    for name in &compiler_names {
        let dest = temp_dir.path().join(name);
        if std::fs::hard_link(&abs_hook, &dest).is_err() {
            // 硬链接失败（如跨盘符），回退到文件拷贝
            std::fs::copy(&abs_hook, &dest)
                .map_err(|e| anyhow!("copy hook-wrapper to {}: {}", dest.display(), e))?;
        }
    }

    // 拼接新 PATH：tempdir;原始PATH
    let orig_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{};{}", temp_dir.path().display(), orig_path);

    println!("Running build command: {}", cmd.join(" "));
    println!("  CPP2RUST_PROJECT_ROOT = {}", abs_project_root.display());
    println!("  CPP2RUST_FEATURE_ROOT = {}", abs_feature_root.display());
    println!("  PATH (prepended)      = {}", temp_dir.path().display());
    println!();

    let status = Command::new(&cmd[0])
        .args(&cmd[1..])
        .current_dir(build_dir)
        .env("PATH", &new_path)
        .env("CPP2RUST_PROJECT_ROOT", &abs_project_root)
        .env("CPP2RUST_FEATURE_ROOT", &abs_feature_root)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| anyhow!("failed to spawn '{}': {}", cmd[0], e))?;

    // temp_dir 在此处 drop，自动清理临时目录（无论成功与否）
    drop(temp_dir);

    if !status.success() {
        return Err(anyhow!(
            "build command failed with exit code {}",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════
//  通用辅助函数（hook_dir / data_dir / dirs_home）— Unix 专用
// ══════════════════════════════════════════════════════════════════════════

/// 从运行中二进制文件所在目录开始向上查找 `hook/` 目录（Unix 专用）。
/// 若找不到，则将嵌入二进制的 hook 源文件解压到用户数据目录，
/// 以便 `cargo install` 用户无需单独检出代码即可使用。
#[cfg(unix)]
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

/// 返回用户 hook 数据目录，不存在或文件过时时创建目录并写入嵌入的 `hook.cpp` / `Makefile`（Unix 专用）。
///
/// 目录路径：
/// - Linux / 其他：`$XDG_DATA_HOME/cpp2rust-demo/hook/`
///   （默认 `~/.local/share/cpp2rust-demo/hook/`）
/// - macOS：`~/Library/Application Support/cpp2rust-demo/hook/`
#[cfg(unix)]
fn ensure_hook_data_dir() -> Result<PathBuf> {
    let base = data_dir().ok_or_else(|| anyhow!("cannot determine user data directory"))?;
    let hook_dir = base.join("cpp2rust-demo").join("hook");

    std::fs::create_dir_all(&hook_dir)
        .map_err(|e| anyhow!("create_dir_all {}: {}", hook_dir.display(), e))?;

    // 若 hook.cpp 不存在或内容有变化，则写入（二进制更新时自动升级）。
    let cpp_path = hook_dir.join("hook.cpp");
    let needs_write = match std::fs::read_to_string(&cpp_path) {
        Ok(existing) => existing != HOOK_CPP,
        Err(_) => true,
    };
    if needs_write {
        std::fs::write(&cpp_path, HOOK_CPP)
            .map_err(|e| anyhow!("write {}: {}", cpp_path.display(), e))?;
    }

    // 若 Makefile 不存在或内容有变化，则写入。
    let mk_path = hook_dir.join("Makefile");
    let mk_needs_write = match std::fs::read_to_string(&mk_path) {
        Ok(existing) => existing != HOOK_MAKEFILE,
        Err(_) => true,
    };
    if mk_needs_write {
        std::fs::write(&mk_path, HOOK_MAKEFILE)
            .map_err(|e| anyhow!("write {}: {}", mk_path.display(), e))?;
    }

    Ok(hook_dir)
}

/// 平台相关的基础数据目录（不含应用子路径）。
fn data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        // Windows：优先 LOCALAPPDATA（C:\Users\<user>\AppData\Local），
        // 回退到 APPDATA（C:\Users\<user>\AppData\Roaming）。
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let p = PathBuf::from(local);
            if p.is_absolute() {
                return Some(p);
            }
        }
        if let Some(roaming) = std::env::var_os("APPDATA") {
            let p = PathBuf::from(roaming);
            if p.is_absolute() {
                return Some(p);
            }
        }
        dirs_home().map(|h| h.join("AppData").join("Local"))
    }
    #[cfg(target_os = "macos")]
    {
        dirs_home().map(|h| h.join("Library").join("Application Support"))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // Linux / 其他：优先使用 XDG_DATA_HOME；回退到 ~/.local/share。
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
fn dirs_home() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        // Windows：优先 USERPROFILE，回退到 HOMEDRIVE+HOMEPATH。
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            let p = PathBuf::from(profile);
            if p.is_absolute() {
                return Some(p);
            }
        }
        let drive = std::env::var_os("HOMEDRIVE").unwrap_or_default();
        let path = std::env::var_os("HOMEPATH").unwrap_or_default();
        let mut full = PathBuf::from(drive);
        full.push(path);
        if full.is_absolute() {
            return Some(full);
        }
        None
    }
    #[cfg(not(windows))]
    {
        // Unix：优先使用 HOME 环境变量（适用于大多数 POSIX 环境）。
        if let Some(h) = std::env::var_os("HOME") {
            let p = PathBuf::from(h);
            if p.is_absolute() {
                return Some(p);
            }
        }
        None
    }
}

// ══════════════════════════════════════════════════════════════════════════
//  测试
// ══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_dir_exists_or_can_be_inferred() {
        // data_dir() 在所有平台上均应返回 Some(...)
        let d = data_dir();
        assert!(d.is_some(), "data_dir() should return Some on all platforms");
    }

    #[test]
    fn test_dirs_home_exists_or_can_be_inferred() {
        let h = dirs_home();
        assert!(h.is_some(), "dirs_home() should return Some on all platforms");
    }

    #[cfg(windows)]
    mod windows_tests {
        use super::super::*;

        #[test]
        fn test_data_dir_windows() {
            // 在 Windows 上，data_dir() 应返回 LOCALAPPDATA 或 APPDATA 路径
            let d = data_dir().expect("data_dir() should return Some on Windows");
            // 路径应是绝对路径
            assert!(d.is_absolute(), "data_dir() should return an absolute path");
        }

        #[test]
        fn test_dirs_home_windows() {
            let h = dirs_home().expect("dirs_home() should return Some on Windows");
            assert!(h.is_absolute(), "dirs_home() should return an absolute path");
        }

        #[test]
        fn test_strip_unc_prefix() {
            let p = std::path::Path::new(r"\\?\C:\Users\test");
            let result = strip_unc_prefix(p);
            assert_eq!(result, std::path::PathBuf::from(r"C:\Users\test"));

            let p2 = std::path::Path::new(r"C:\Users\test");
            assert_eq!(strip_unc_prefix(p2), std::path::PathBuf::from(r"C:\Users\test"));
        }

        #[test]
        fn test_ensure_hook_wrapper_exe() {
            // 验证 ensure_hook_wrapper_exe() 能正确解压文件（需要写入权限）
            let result = ensure_hook_wrapper_exe();
            assert!(result.is_ok(), "ensure_hook_wrapper_exe() should succeed: {:?}", result);
            let path = result.unwrap();
            assert!(path.exists(), "hook-wrapper.exe should exist at {}", path.display());
            assert!(path.extension().map(|e| e == "exe").unwrap_or(false));
        }

        #[test]
        fn test_hook_wrapper_exe_is_valid() {
            // 验证解压的 hook-wrapper.exe 与嵌入的字节匹配
            let path = ensure_hook_wrapper_exe().expect("should succeed");
            let on_disk = std::fs::read(&path).expect("should be readable");
            assert_eq!(on_disk, HOOK_WRAPPER_EXE, "on-disk bytes should match embedded bytes");
        }
    }
}
