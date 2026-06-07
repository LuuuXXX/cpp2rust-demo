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
    let raw_args: Vec<String> = env::args().skip(1).collect();
    let kind = CompilerKind::from_env();

    let real_cc = env::var("CPP2RUST_REAL_CC").unwrap_or_else(|_| "g++".to_string());

    // 将 MSYS2/Cygwin POSIX 路径规范化为 Windows 路径（如 /d/a/... → D:\a\...）。
    // 背景：MSYS2 bash 在调用原生 Windows 可执行文件时，有时不转换路径参数（特别是
    // 当 shim 被 MSYS2 识别为 MSYS2-aware 程序时）。若 shim 直接把 POSIX 路径转发
    // 给真实 g++，MinGW cc1plus.exe 无法正确解析，导致"\\filename.cpp"报错。
    let args: Vec<String> = raw_args.iter().map(|a| normalize_arg(a)).collect();

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
///
/// MSVC 已知选项前缀白名单（大小写不敏感）：
///   /c /C /P /E /EP /TC /TP /Fo /Fe /Fi /Fd /Fp /FA /Fa /FR /Fr
///   /I /D /U /FI /FU /EH /MD /MT /LD /LDd /MDd /MTd /W0-W4 /WX /GR /GX
///   /O1 /O2 /Ob /Oi /Os /Ot /Ox /Oy /GL /GS /RTC /Zi /ZI /Z7 /Zo
///   /std /arch /Zc /permissive /nologo /showIncludes /source-charset /utf-8
///
/// 规则：
/// 1. 形如 `X:/...`（X 为字母）的是 Windows 绝对路径 → true
/// 2. 若不匹配任何已知前缀，则保守地视为路径 → true
fn looks_like_file_path(s: &str) -> bool {
    let bytes = s.as_bytes();
    // Windows 绝对路径：C:/... 或 C:\...（第二个字符为 ':'）
    if bytes.len() >= 3 && bytes[1] == b':' {
        return true;
    }
    // MSYS2/Cygwin POSIX 驱动器路径：/c/... /d/... 等（/ + 小写字母 + /）
    // 需要在 MSVC 选项前缀匹配之前检测，避免 /d/a/... 误匹配 MSVC /D 宏定义选项
    if bytes.len() >= 3
        && bytes[0] == b'/'
        && bytes[1].is_ascii_lowercase()
        && bytes[2] == b'/'
    {
        return true;
    }
    // 已知 MSVC 选项前缀列表（均以 '/' 开头，大小写不敏感）
    let known_flags: &[&str] = &[
        "/c", "/C", "/P", "/E", "/EP", "/TC", "/TP",
        "/Fo", "/Fe", "/Fi", "/Fd", "/Fp", "/FA", "/Fa", "/FR", "/Fr",
        "/I", "/D", "/U", "/FI", "/FU",
        "/EH", "/MD", "/MT", "/LD", "/LDd", "/MDd", "/MTd",
        "/W", "/WX", "/GR", "/GX",
        "/O", "/GL", "/GS", "/RTC",
        "/Zi", "/ZI", "/Z7", "/Zo",
        "/std", "/arch", "/Zc", "/permissive", "/nologo",
        "/showIncludes", "/source-charset", "/utf-8",
    ];
    let lower = s.to_ascii_lowercase();
    for flag in known_flags {
        if lower.starts_with(&flag.to_ascii_lowercase()) {
            return false; // 匹配已知选项 → 不是路径
        }
    }
    // 未知 "/" 开头字符串，保守视为路径
    true
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
    // /P: 预处理到文件；/C: 保留注释；/Fi"<path>": 输出路径（带引号防路径含空格）
    cmd.arg("/P").arg("/C");
    let out_str = out_path.to_string_lossy();
    cmd.arg(format!("/Fi\"{}\"", out_str));

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
    // 优先使用 canonicalize；若失败（例如 POSIX 路径 /d/a/... 在 Windows 上），
    // 尝试将 MSYS2 POSIX 驱动器路径手动转换为 Windows 路径后再 canonicalize。
    let abs_src = src
        .canonicalize()
        .ok()
        .or_else(|| msys2_to_windows(src).and_then(|p| p.canonicalize().ok()))
        .or_else(|| msys2_to_windows(src))?;

    // project_root 与 feature_root 可能带有 \\?\ 前缀（Windows 扩展路径格式），
    // 需要去掉才能与普通 abs_src 做前缀匹配。
    let normal_root = strip_verbatim(project_root);
    let normal_src = strip_verbatim(&abs_src);

    let rel = normal_src.strip_prefix(&normal_root).ok()?;

    let normal_feat = strip_verbatim(feature_root);
    let out_path = normal_feat.join(rel).with_extension("cpp2rust");
    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    Some((abs_src, out_path))
}

/// 去掉 Windows 扩展路径前缀 `\\?\`，返回普通路径。
/// 若无前缀则返回原路径的 PathBuf 拷贝。
fn strip_verbatim(p: &Path) -> PathBuf {
    match p.to_str() {
        Some(s) if s.starts_with("\\\\?\\") => PathBuf::from(&s[4..]),
        _ => p.to_path_buf(),
    }
}

/// 将 MSYS2/Cygwin POSIX 驱动器路径（如 `/d/a/foo/bar`）转换为
/// Windows 路径（如 `D:\a\foo\bar`）。
/// 仅处理以 `/<小写字母>/` 开头的路径；其他形式返回 None。
fn msys2_to_windows(p: &Path) -> Option<PathBuf> {
    let s = p.to_str()?;
    let bytes = s.as_bytes();
    // 形如 /d/a/... 或 /d/
    if bytes.len() >= 3
        && bytes[0] == b'/'
        && bytes[1].is_ascii_lowercase()
        && bytes[2] == b'/'
    {
        let drive = (bytes[1] as char).to_ascii_uppercase();
        // bytes[2..] 以 '/' 开头，例如 "/a/foo/bar"
        let rest = s[2..].replace('/', "\\");
        Some(PathBuf::from(format!("{}:{}", drive, rest)))
    } else {
        None
    }
}

/// 将参数中的 MSYS2 POSIX 路径转换为 Windows 路径。
///
/// 处理以下格式：
/// - 独立路径参数：`/d/a/foo/bar.cpp` → `D:\a\foo\bar.cpp`
/// - `-I/d/a/foo` 形式（GCC include 标志）：保留 `-I` 前缀，转换路径部分
/// - 其他参数保持不变
fn normalize_arg(s: &str) -> String {
    let bytes = s.as_bytes();

    // 独立的 POSIX 驱动器路径：/d/a/...
    if bytes.len() >= 3
        && bytes[0] == b'/'
        && bytes[1].is_ascii_lowercase()
        && bytes[2] == b'/'
    {
        if let Some(p) = msys2_to_windows(Path::new(s)) {
            return p.to_string_lossy().into_owned();
        }
    }

    // -I/d/a/... 形式（-I 后紧跟 POSIX 路径）
    if bytes.len() >= 5
        && bytes[0] == b'-'
        && bytes[1] == b'I'
        && bytes[2] == b'/'
        && bytes[3].is_ascii_lowercase()
        && bytes[4] == b'/'
    {
        let path_part = &s[2..];
        if let Some(p) = msys2_to_windows(Path::new(path_part)) {
            return format!("-I{}", p.to_string_lossy());
        }
    }

    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CompilerKind::is_compile_mode ────────────────────────────────

    #[test]
    fn gnu_compile_mode_detects_dash_c() {
        let kind = CompilerKind::Gnu;
        assert!(
            kind.is_compile_mode(&["-c".to_string()]),
            "GNU: -c 应触发编译模式"
        );
    }

    #[test]
    fn gnu_compile_mode_in_multiarg_list() {
        let kind = CompilerKind::Gnu;
        let args: Vec<String> = ["-I/usr/include", "-o", "foo.o", "foo.cpp", "-c"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            kind.is_compile_mode(&args),
            "GNU: 多参数列表中含 -c 应触发编译模式"
        );
    }

    #[test]
    fn gnu_link_mode_no_dash_c() {
        let kind = CompilerKind::Gnu;
        let args: Vec<String> = ["-o", "prog", "foo.o", "bar.o"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(
            !kind.is_compile_mode(&args),
            "GNU: 不含 -c 不应触发编译模式"
        );
    }

    #[test]
    fn gnu_compile_mode_empty_args() {
        let kind = CompilerKind::Gnu;
        assert!(
            !kind.is_compile_mode(&[]),
            "GNU: 空参数列表不应触发编译模式"
        );
    }

    #[test]
    fn msvc_compile_mode_detects_slash_c() {
        let kind = CompilerKind::Msvc;
        assert!(
            kind.is_compile_mode(&["/c".to_string()]),
            "MSVC: /c 应触发编译模式"
        );
    }

    #[test]
    fn msvc_slash_c_uppercase_not_compile_mode() {
        // `/C` 是 MSVC 预处理选项"保留注释"（preprocessing comment-preserve），
        // 不是"编译（不链接）"模式标志 `/c`（两者大小写不同，语义完全不同）
        let kind = CompilerKind::Msvc;
        assert!(
            !kind.is_compile_mode(&["/C".to_string()]),
            "MSVC: /C (大写) 不应触发编译模式"
        );
    }

    #[test]
    fn msvc_gnu_dash_c_not_compile_mode() {
        // MSVC 编译器不识别 `-c`，is_compile_mode 应返回 false
        let kind = CompilerKind::Msvc;
        assert!(
            !kind.is_compile_mode(&["-c".to_string()]),
            "MSVC: GNU 风格的 -c 不应触发 MSVC 编译模式"
        );
    }

    // ── looks_like_file_path ─────────────────────────────────────────

    #[test]
    fn absolute_windows_path_is_file_path() {
        assert!(
            looks_like_file_path("C:/Users/foo/bar.cpp"),
            "C:/... 形式应被视为文件路径"
        );
        assert!(
            looks_like_file_path("D:/project/src/main.cpp"),
            "D:/... 形式应被视为文件路径"
        );
    }

    #[test]
    fn known_msvc_flags_not_file_path() {
        for flag in &[
            "/c", "/C", "/P", "/E", "/EP", "/TC", "/TP",
            "/Fo", "/Fe", "/Fi", "/Fd",
            "/I", "/D", "/U", "/FI",
            "/EH", "/MD", "/MT", "/WX", "/GR",
            "/O1", "/O2", "/GL", "/GS",
            "/Zi", "/ZI", "/Z7",
            "/std:c++17", "/nologo",
        ] {
            assert!(
                !looks_like_file_path(flag),
                "已知 MSVC 选项 {} 不应被视为文件路径",
                flag
            );
        }
    }

    #[test]
    fn unknown_slash_prefix_treated_as_path() {
        // 未知的 /xxx 选项保守视为路径（/s 不是小写字母开头的驱动器路径，走保守分支）
        assert!(
            looks_like_file_path("/some/unknown/path"),
            "未知 /xxx 前缀应保守视为文件路径"
        );
    }

    #[test]
    fn posix_drive_path_is_file_path() {
        // MSYS2 POSIX 驱动器路径：/d/a/... 不应被误识别为 MSVC /D 宏定义选项
        assert!(
            looks_like_file_path("/d/a/project/src/main.cpp"),
            "/d/a/... POSIX 路径应被视为文件路径，而非 MSVC /D 宏定义选项"
        );
        assert!(
            looks_like_file_path("/c/Users/runner/project/foo.cpp"),
            "/c/... POSIX 路径应被视为文件路径"
        );
    }

    // ── normalize_arg ────────────────────────────────────────────────

    #[test]
    fn normalize_arg_posix_path_to_windows() {
        let result = normalize_arg("/d/a/project/src/main.cpp");
        assert_eq!(result, r"D:\a\project\src\main.cpp");
    }

    #[test]
    fn normalize_arg_include_posix_to_windows() {
        let result = normalize_arg("/d/a/project/include");
        assert_eq!(result, r"D:\a\project\include");
    }

    #[test]
    fn normalize_arg_dash_i_posix_to_windows() {
        let result = normalize_arg("-I/d/a/project/include");
        assert_eq!(result, r"-ID:\a\project\include");
    }

    #[test]
    fn normalize_arg_non_posix_unchanged() {
        // 普通 Windows 路径或标志不应被修改
        assert_eq!(normalize_arg("-c"), "-c");
        assert_eq!(normalize_arg("-std=c++17"), "-std=c++17");
        assert_eq!(
            normalize_arg(r"D:\a\project\main.cpp"),
            r"D:\a\project\main.cpp"
        );
    }

    // ── msys2_to_windows ─────────────────────────────────────────────

    #[test]
    fn msys2_to_windows_basic() {
        let p = Path::new("/d/a/foo/bar.cpp");
        let result = msys2_to_windows(p).unwrap();
        assert_eq!(result, PathBuf::from(r"D:\a\foo\bar.cpp"));
    }

    #[test]
    fn msys2_to_windows_c_drive() {
        let p = Path::new("/c/Users/runner/project/main.cpp");
        let result = msys2_to_windows(p).unwrap();
        assert_eq!(result, PathBuf::from(r"C:\Users\runner\project\main.cpp"));
    }

    #[test]
    fn msys2_to_windows_non_posix_returns_none() {
        // 非 POSIX 驱动器路径不应被转换
        assert!(msys2_to_windows(Path::new("-c")).is_none());
        assert!(msys2_to_windows(Path::new(r"D:\a\foo.cpp")).is_none());
    }

    // ── strip_verbatim ───────────────────────────────────────────────

    #[test]
    fn strip_verbatim_removes_prefix() {
        let p = PathBuf::from(r"\\?\D:\a\project");
        let result = strip_verbatim(&p);
        assert_eq!(result, PathBuf::from(r"D:\a\project"));
    }

    #[test]
    fn strip_verbatim_no_prefix_unchanged() {
        let p = PathBuf::from(r"D:\a\project");
        let result = strip_verbatim(&p);
        assert_eq!(result, PathBuf::from(r"D:\a\project"));
    }
}
