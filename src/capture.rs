use crate::error::Result;
use anyhow::anyhow;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// 将 hook 源文件内容直接嵌入 binary，确保 `cargo install` 后无需额外文件。
const HOOK_CPP: &str = include_str!("../hook/hook.cpp");
const HOOK_MAKEFILE: &str = include_str!("../hook/Makefile");

/// 从 `hook/Makefile` 构建当前平台的 hook 动态库。
///
/// 若 hook 动态库已存在且比 `hook.cpp` 更新，则跳过编译（快速路径）。
/// 返回 hook 动态库的路径。
pub fn build_hook() -> Result<PathBuf> {
    let hook_dir = hook_dir()?;
    let so = hook_dir.join(hook_library_name());
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
            "hook library not found after build at {}",
            so.display()
        ));
    }

    println!("Hook library built: {}", so.display());
    Ok(so)
}

/// 注入 hook 动态库并执行用户提供的构建命令。
pub fn run_with_hook(
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
    println!(
        "  {}            = {}",
        hook_injection_env(),
        abs_hook.display()
    );
    println!();

    #[cfg(target_os = "macos")]
    capture_direct_compiler_command(build_dir, cmd, &abs_project_root, &abs_feature_root)?;

    let mut command = Command::new(&cmd[0]);
    command
        .args(&cmd[1..])
        .current_dir(build_dir)
        .env(hook_injection_env(), &abs_hook)
        .env("CPP2RUST_PROJECT_ROOT", &abs_project_root)
        .env("CPP2RUST_FEATURE_ROOT", &abs_feature_root)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    #[cfg(target_os = "macos")]
    command.env("DYLD_FORCE_FLAT_NAMESPACE", "1");

    let status = command
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

#[cfg(target_os = "macos")]
fn capture_direct_compiler_command(
    build_dir: &Path,
    cmd: &[String],
    project_root: &Path,
    feature_root: &Path,
) -> Result<()> {
    if !is_cpp_compiler(&cmd[0]) {
        return Ok(());
    }

    let parsed = parse_compiler_command(build_dir, &cmd[1..]);
    if parsed.cpp_files.is_empty() {
        return Ok(());
    }

    for cpp_file in parsed.cpp_files {
        preprocess_direct_cpp_file(
            build_dir,
            &cmd[0],
            &parsed.preprocess_args,
            &cpp_file,
            project_root,
            feature_root,
        )?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
struct ParsedCompilerCommand {
    preprocess_args: Vec<String>,
    cpp_files: Vec<PathBuf>,
}

#[cfg(target_os = "macos")]
fn parse_compiler_command(build_dir: &Path, args: &[String]) -> ParsedCompilerCommand {
    let mut preprocess_args = Vec::new();
    let mut cpp_files = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if arg == "-o" {
            i += 2;
            continue;
        }
        if arg.starts_with("-o") && arg.len() > 2 {
            i += 1;
            continue;
        }

        if arg == "-include" || arg == "-isystem" || arg == "-iquote" {
            preprocess_args.push(arg.clone());
            if let Some(value) = args.get(i + 1) {
                preprocess_args.push(value.clone());
            }
            i += 2;
            continue;
        }

        if is_preserved_preprocess_arg(arg) {
            preprocess_args.push(arg.clone());
            if (arg == "-I" || arg == "-D" || arg == "-U") && args.get(i + 1).is_some() {
                i += 1;
                preprocess_args.push(args[i].clone());
            }
            i += 1;
            continue;
        }

        if !arg.starts_with('-') && is_cpp_file(arg) {
            let path = build_dir.join(arg);
            if path.is_file() {
                cpp_files.push(path);
            }
        }

        i += 1;
    }

    ParsedCompilerCommand {
        preprocess_args,
        cpp_files,
    }
}

#[cfg(target_os = "macos")]
fn preprocess_direct_cpp_file(
    build_dir: &Path,
    compiler: &str,
    preprocess_args: &[String],
    cpp_file: &Path,
    project_root: &Path,
    feature_root: &Path,
) -> Result<()> {
    let cpp_file = cpp_file
        .canonicalize()
        .map_err(|e| anyhow!("canonicalize {}: {}", cpp_file.display(), e))?;
    let rel = cpp_file.strip_prefix(project_root).map_err(|_| {
        anyhow!(
            "{} is not under {}",
            cpp_file.display(),
            project_root.display()
        )
    })?;
    let out = feature_root.join("c").join(rel).with_extension(format!(
        "{}cpp2rust",
        rel.extension()
            .and_then(OsStr::to_str)
            .map(|ext| format!("{}.", ext))
            .unwrap_or_default()
    ));

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow!("create dir {}: {}", parent.display(), e))?;
    }

    let opts = out.with_extension(format!(
        "{}opts",
        out.extension()
            .and_then(OsStr::to_str)
            .map(|ext| format!("{}.", ext))
            .unwrap_or_default()
    ));
    std::fs::write(&opts, quote_args(preprocess_args))
        .map_err(|e| anyhow!("write {}: {}", opts.display(), e))?;

    let status = Command::new(compiler)
        .arg("-E")
        .arg("-C")
        .arg(&cpp_file)
        .arg("-o")
        .arg(&out)
        .args(preprocess_args)
        .current_dir(build_dir)
        .status()
        .map_err(|e| anyhow!("failed to preprocess {}: {}", cpp_file.display(), e))?;

    if !status.success() {
        return Err(anyhow!(
            "preprocess failed for {} with exit code {}",
            cpp_file.display(),
            status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn is_cpp_compiler(command: &str) -> bool {
    let Some(name) = Path::new(command).file_name().and_then(OsStr::to_str) else {
        return false;
    };
    ["g++", "clang++", "c++"].iter().any(|compiler| {
        name == *compiler
            || name
                .strip_prefix(compiler)
                .is_some_and(|rest| rest.starts_with('-'))
    })
}

#[cfg(target_os = "macos")]
fn is_cpp_file(path: &str) -> bool {
    ["cpp", "cc", "cxx", "c++", "C", "cp"]
        .iter()
        .any(|ext| path.ends_with(&format!(".{ext}")))
}

#[cfg(target_os = "macos")]
fn is_preserved_preprocess_arg(arg: &str) -> bool {
    arg == "-I"
        || arg == "-D"
        || arg == "-U"
        || arg.starts_with("-I")
        || arg.starts_with("-D")
        || arg.starts_with("-U")
        || arg.starts_with("-std=")
        || arg == "-fshort-enums"
}

#[cfg(target_os = "macos")]
fn quote_args(args: &[String]) -> String {
    args.iter()
        .map(|arg| format!("\"{}\"", arg.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

fn hook_library_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "libhook.dylib"
    } else {
        "libhook.so"
    }
}

fn hook_injection_env() -> &'static str {
    if cfg!(target_os = "macos") {
        "DYLD_INSERT_LIBRARIES"
    } else {
        "LD_PRELOAD"
    }
}

/// 从运行中二进制文件所在目录开始向上查找 `hook/` 目录。
/// 若找不到，则将嵌入二进制的 hook 源文件解压到用户数据目录，
/// 以便 `cargo install` 用户无需单独检出代码即可使用。
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

/// 返回用户 hook 数据目录，不存在或文件过时时创建目录并写入嵌入的 `hook.cpp` / `Makefile`。
///
/// 目录路径：
/// - Linux / 其他：`$XDG_DATA_HOME/cpp2rust-demo/hook/`
///   （默认 `~/.local/share/cpp2rust-demo/hook/`）
/// - macOS：`~/Library/Application Support/cpp2rust-demo/hook/`
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
    #[cfg(target_os = "macos")]
    {
        dirs_home().map(|h| h.join("Library").join("Application Support"))
    }
    #[cfg(not(target_os = "macos"))]
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
/// 使用 `HOME` 环境变量，这是标准的 POSIX 机制，
/// 适用于 Linux、macOS 及大多数类 Unix 系统。
/// 本工具依赖 Unix 动态库注入机制，不支持 Windows，因此无需 Windows 专用的回退逻辑。
fn dirs_home() -> Option<PathBuf> {
    // 优先使用 HOME 环境变量（适用于大多数 POSIX 环境）。
    if let Some(h) = std::env::var_os("HOME") {
        let p = PathBuf::from(h);
        if p.is_absolute() {
            return Some(p);
        }
    }
    None
}
