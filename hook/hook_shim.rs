// hook_shim.rs — Windows 编译器 Shim，用于替代 Linux/macOS 的 LD_PRELOAD 捕获机制。
//
// 工作原理：
//   1. capture::build_hook_windows() 将本文件内容写入临时目录并用 rustc 编译为 hook_shim.exe
//   2. capture::run_with_hook_windows() 将 hook_shim.exe 复制到另一个临时目录，
//      以真实编译器的基名（如 g++.exe / cl.exe）命名，然后将该目录插入 PATH 最前面。
//   3. 构建系统调用编译器时实际触发本 shim：
//      a. 将所有参数原样转发给真实编译器（通过 CPP2RUST_REAL_CC 找到）完成正常编译。
//      b. 若参数含编译标志（GCC: -c；MSVC: /c），额外运行预处理器生成 .cpp2rust 捕获文件。
//
// 环境变量：
//   CPP2RUST_REAL_CC        — 真实编译器的完整路径（如 C:\msys64\mingw64\bin\g++.exe）
//   CPP2RUST_PROJECT_ROOT   — 用户 C++ 项目根目录（用于计算相对路径）
//   CPP2RUST_FEATURE_ROOT   — .cpp2rust/<feature>/ 输出目录
//   CPP2RUST_COMPILER_KIND  — 编译器类型："gnu"（默认）或 "msvc"（cl.exe）

use std::env;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

/// 编译器类型，由 CPP2RUST_COMPILER_KIND 环境变量设置。
enum CompilerKind {
    Gnu,
    Msvc,
}

impl CompilerKind {
    fn from_env() -> Self {
        match env::var("CPP2RUST_COMPILER_KIND")
            .as_deref()
            .unwrap_or("gnu")
        {
            "msvc" => CompilerKind::Msvc,
            _ => CompilerKind::Gnu,
        }
    }

    /// 检测给定参数列表是否为"编译（不链接）"模式。
    fn is_compile_mode(&self, args: &[String]) -> bool {
        match self {
            // GCC/Clang: `-c`
            CompilerKind::Gnu => args.iter().any(|a| a == "-c"),
            // MSVC: `/c`（注意：`/C` 是"保留注释"，两者不同）
            CompilerKind::Msvc => args.iter().any(|a| a == "/c"),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let kind = CompilerKind::from_env();

    let real_cc = env::var("CPP2RUST_REAL_CC").unwrap_or_else(|_| "g++".to_string());

    // ── 步骤 1：原样转发给真实编译器 ──
    let status = Command::new(&real_cc)
        .args(&args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("[cpp2rust-shim] failed to run '{}': {}", real_cc, e);
            exit(1);
        });

    let exit_code = status.code().unwrap_or(1);
    if exit_code != 0 {
        exit(exit_code);
    }

    // ── 步骤 2：若为编译模式，额外生成 .cpp2rust 捕获文件 ──
    if !kind.is_compile_mode(&args) {
        exit(0);
    }

    let project_root = match env::var("CPP2RUST_PROJECT_ROOT") {
        Ok(v) => PathBuf::from(v),
        Err(_) => exit(0), // 未处于捕获模式，正常退出
    };
    let feature_root = match env::var("CPP2RUST_FEATURE_ROOT") {
        Ok(v) => PathBuf::from(v),
        Err(_) => exit(0),
    };

    // 遍历参数，找到所有 .cpp/.cc/.cxx/.C 输入文件并分别预处理
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];

        // 跳过选项标志及其参数（GCC 风格，以 '-' 开头）
        if arg.starts_with('-') {
            // 带值的选项（如 -o <file>、-I <dir>）跳过下一个 token
            if matches!(
                arg.as_str(),
                "-o" | "-I"
                    | "-isystem"
                    | "-include"
                    | "-MF"
                    | "-MT"
                    | "-MQ"
                    | "-D"
                    | "-U"
                    | "-idirafter"
                    | "-imacros"
                    | "-L"
                    | "-l"
            ) {
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        // 跳过 MSVC 风格选项（以 '/' 开头，且不是文件路径）
        if arg.starts_with('/') && !looks_like_file_path(arg) {
            // 带值的 MSVC 选项：/I <dir>、/D <macro>、/FI <file>、/Fo <obj>
            if matches!(arg.as_str(), "/I" | "/D" | "/FI" | "/Fo" | "/Fe" | "/Fd") {
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        let p = Path::new(arg.as_str());
        if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            if matches!(ext, "cpp" | "cc" | "cxx" | "C") {
                match kind {
                    CompilerKind::Gnu => {
                        preprocess_file_gnu(&real_cc, p, &args, &project_root, &feature_root);
                    }
                    CompilerKind::Msvc => {
                        preprocess_file_msvc(&real_cc, p, &args, &project_root, &feature_root);
                    }
                }
            }
        }
        i += 1;
    }

    exit(0);
}

/// 判断一个以 '/' 开头的字符串是否更像文件路径而非 MSVC 选项。
/// 例如 `C:/foo/bar.cpp` 是路径，`/c` `/I` 是选项。
fn looks_like_file_path(s: &str) -> bool {
    // Windows 绝对路径形如 C:/... 或 /foo/bar（POSIX 风格）
    // 简单启发式：若第二个字符是 ':' 或第一个 '/' 之后紧跟另一个 '/'，则为路径
    let bytes = s.as_bytes();
    if bytes.len() >= 3 && bytes[1] == b':' {
        return true; // C:/... D:/...
    }
    // 单字符 '/' 后跟字母+数字组合视为 MSVC flag，否则视为路径
    // 粗略判断：若 '/' 后的第一个字符是大写或小写字母，且长度较短，则为 flag
    if bytes.len() <= 8 {
        return false; // 短字符串大概率是 flag
    }
    // 较长的 /xxx/yyy/zzz 可能是路径
    s.contains('/') && s.len() > 8
}

// ─────────────────────────────────────────────────────────────────
//  GCC / Clang 预处理（-E -C）
// ─────────────────────────────────────────────────────────────────

/// 对单个 C++ 源文件运行 GCC/Clang 预处理器（-E -C），
/// 将结果写入 feature_root 下对应的 .cpp2rust 文件。
fn preprocess_file_gnu(
    cc: &str,
    src: &Path,
    all_args: &[String],
    project_root: &Path,
    feature_root: &Path,
) {
    let (abs_src, out_path) = match resolve_paths(src, project_root, feature_root) {
        Some(v) => v,
        None => return,
    };

    // 从原始参数中收集 -I / -D / -std / -isystem 传递给预处理器
    let mut cmd = Command::new(cc);
    cmd.arg("-E").arg("-C");
    let mut skip_next = false;
    for a in all_args {
        if skip_next {
            cmd.arg(a);
            skip_next = false;
            continue;
        }
        if a == "-I" || a == "-isystem" || a == "-include" {
            cmd.arg(a);
            skip_next = true;
        } else if a.starts_with("-I")
            || a.starts_with("-D")
            || a.starts_with("-std")
            || a.starts_with("-isystem")
        {
            cmd.arg(a);
        }
    }
    cmd.arg(&abs_src).arg("-o").arg(&out_path);

    let _ = cmd.status(); // 预处理失败不影响正常构建
}

// ─────────────────────────────────────────────────────────────────
//  MSVC 预处理（/P /C /Fi<output>）
// ─────────────────────────────────────────────────────────────────

/// 对单个 C++ 源文件运行 MSVC 预处理器（cl /P /C），
/// 将结果写入 feature_root 下对应的 .cpp2rust 文件。
///
/// MSVC 预处理标志说明：
///   /P         — 将预处理结果写入文件（而非 stdout）
///   /C         — 保留注释（等价于 gcc -C）
///   /Fi<path>  — 指定预处理输出文件路径（/P 才有效；/Fi 与路径之间无空格）
///   /I <dir>   — 添加包含路径
///   /D <macro> — 定义宏
///   /std:c++17 — 指定 C++ 标准
fn preprocess_file_msvc(
    cc: &str,
    src: &Path,
    all_args: &[String],
    project_root: &Path,
    feature_root: &Path,
) {
    let (abs_src, out_path) = match resolve_paths(src, project_root, feature_root) {
        Some(v) => v,
        None => return,
    };

    let mut cmd = Command::new(cc);
    // /P: 预处理到文件；/C: 保留注释；/Fi<path>: 输出路径（无空格）
    cmd.arg("/P").arg("/C");
    cmd.arg(format!("/Fi{}", out_path.display()));

    // 从原始参数中收集 MSVC 风格的包含路径、宏定义、C++ 标准
    let mut skip_next = false;
    for a in all_args {
        if skip_next {
            cmd.arg(a);
            skip_next = false;
            continue;
        }
        // /I <dir>（带空格的形式）
        if a == "/I" {
            cmd.arg(a);
            skip_next = true;
            continue;
        }
        // /D <macro>（带空格的形式）
        if a == "/D" {
            cmd.arg(a);
            skip_next = true;
            continue;
        }
        // /I<dir>、/D<macro>、/std:c++xx（无空格形式）
        if a.starts_with("/I")
            || a.starts_with("/D")
            || a.starts_with("/std:")
            || a.starts_with("/FI") // forced include
        {
            cmd.arg(a);
            continue;
        }
        // GCC 风格的 -I / -D / -std（当 MSVC 与 CMake 混用时偶有出现）
        if a == "-I" || a == "-isystem" || a == "-include" {
            cmd.arg(a);
            skip_next = true;
            continue;
        }
        if a.starts_with("-I") || a.starts_with("-D") || a.starts_with("-std") {
            cmd.arg(a);
        }
    }
    cmd.arg(&abs_src);

    let _ = cmd.status(); // 预处理失败不影响正常构建
}

// ─────────────────────────────────────────────────────────────────
//  共用：路径解析
// ─────────────────────────────────────────────────────────────────

/// 将源文件路径规范化，计算相对路径，并确定输出路径。
/// 返回 `(abs_src, out_path)`，若不在 project_root 下则返回 None。
fn resolve_paths(
    src: &Path,
    project_root: &Path,
    feature_root: &Path,
) -> Option<(PathBuf, PathBuf)> {
    let abs_src = src.canonicalize().ok()?;
    let rel = abs_src.strip_prefix(project_root).ok()?;
    let out_path = feature_root.join(rel).with_extension("cpp2rust");
    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    Some((abs_src, out_path))
}
