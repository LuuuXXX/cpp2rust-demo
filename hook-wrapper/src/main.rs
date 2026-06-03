/*
 * hook-wrapper — Windows 编译拦截器
 *
 * 工作原理：
 *   cpp2rust-demo 在 Windows 上通过在 PATH 首位注入一个临时目录来拦截编译器调用。
 *   临时目录中有若干以目标编译器命名的本程序副本（cl.exe / clang-cl.exe / g++.exe 等）。
 *   构建系统调用"编译器"时实际执行的是本程序；本程序：
 *     1. 读取 argv[0] 文件名，识别被当作哪个编译器调用
 *     2. 在 PATH 中跳过自身所在目录，定位真实编译器
 *     3. 若是编译器调用且有 C++ 源文件参数，运行预处理保存 .cpp2rust 文件
 *     4. 将所有参数透传给真实编译器，确保构建正常完成
 *
 * 对应环境变量：
 *   CPP2RUST_PROJECT_ROOT   — 工程根目录（必须）
 *   CPP2RUST_FEATURE_ROOT   — feature 输出目录（必须）
 *   CPP2RUST_CC             — 覆盖编译器名称检测（可选）
 *   CPP2RUST_CC_SKIP        — 非空时跳过预处理（防止递归）
 *   CPP2RUST_DEBUG          — 非空时输出调试信息到 stderr
 */

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

// 已知 C++ 编译器名称（不含 .exe 后缀，比较时忽略）
static CC_NAMES: &[&str] = &[
    "cl",
    "clang-cl",
    "g++",
    "clang++",
    "c++",
    "g++-12",
    "g++-13",
    "g++-14",
];

fn debug(msg: &str) {
    if env::var_os("CPP2RUST_DEBUG").is_some() {
        eprintln!("[cpp2rust-hook] {}", msg);
    }
}

/// 去掉 Windows 路径的 `\\?\` 长路径前缀。
fn strip_unc_prefix(p: &Path) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        p.to_path_buf()
    }
}

/// 规范化路径（去掉 \\?\ 前缀）。
fn canonical(p: &Path) -> PathBuf {
    match p.canonicalize() {
        Ok(c) => strip_unc_prefix(&c),
        Err(_) => p.to_path_buf(),
    }
}

/// 是否为已知 C++ 编译器名称（忽略 .exe 后缀，允许版本后缀如 g++-13）。
fn is_known_compiler(name: &str) -> bool {
    // 去掉 .exe 后缀
    let name = if let Some(n) = name.strip_suffix(".exe") {
        n
    } else {
        name
    };
    // 检查自定义覆盖
    if let Ok(cc) = env::var("CPP2RUST_CC") {
        let cc_base = Path::new(&cc)
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or(&cc);
        return name.eq_ignore_ascii_case(cc_base);
    }
    for known in CC_NAMES {
        if name.eq_ignore_ascii_case(known) {
            return true;
        }
        // 支持带版本后缀：g++-13 等（若 known 是前缀，后面跟 '-' + 数字）
        if let Some(rest) = name.strip_prefix(known) {
            if rest.len() > 1 && rest.starts_with('-') && rest[1..].chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                return true;
            }
        }
    }
    false
}

/// 编译器风格分类
#[derive(Debug, Clone, Copy, PartialEq)]
enum CompilerStyle {
    /// MSVC 风格：cl.exe / clang-cl.exe（参数以 `/` 开头）
    Msvc,
    /// GNU 风格：g++.exe / clang++.exe / MinGW（参数以 `-` 开头）
    Gnu,
}

fn compiler_style(name: &str) -> CompilerStyle {
    let name_lc = name.to_ascii_lowercase();
    let stem = if let Some(n) = name_lc.strip_suffix(".exe") {
        n
    } else {
        &name_lc
    };
    if stem == "cl" || stem == "clang-cl" {
        CompilerStyle::Msvc
    } else {
        CompilerStyle::Gnu
    }
}

/// 判断文件名是否为 C++ 源文件扩展名。
fn is_cpp_file(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let lower = lower.replace('\\', "/");
    matches!(
        lower.rsplit('.').next().unwrap_or(""),
        "cpp" | "cc" | "cxx" | "c++" | "cp"
    )
    // 注意：.C（大写）在 case-insensitive Windows 文件系统上等价于 .c，不视为 C++
}

/// 判断一个参数是否是 C++ 源文件（存在于磁盘上）。
fn arg_is_cppfile(arg: &str, style: CompilerStyle) -> bool {
    match style {
        // MSVC: 不以 / 或 - 开头的参数才可能是源文件
        CompilerStyle::Msvc => {
            if arg.starts_with('/') || arg.starts_with('-') {
                return false;
            }
            if !is_cpp_file(arg) {
                return false;
            }
            Path::new(arg).exists()
        }
        // GNU: 不以 - 开头的参数才可能是源文件
        CompilerStyle::Gnu => {
            if arg.starts_with('-') {
                return false;
            }
            if !is_cpp_file(arg) {
                return false;
            }
            Path::new(arg).exists()
        }
    }
}

/// 从编译器参数中提取预处理所需的 flags（-I/-D/-U/-std 等）。
fn extract_preprocess_flags(args: &[String], style: CompilerStyle) -> Vec<String> {
    let mut flags = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match style {
            CompilerStyle::Msvc => {
                // /I<dir> 或 /I <dir>
                if arg.eq_ignore_ascii_case("/I") || arg.eq_ignore_ascii_case("-I") {
                    flags.push(arg.clone());
                    if i + 1 < args.len() {
                        i += 1;
                        flags.push(args[i].clone());
                    }
                } else if arg.len() > 2
                    && (arg.starts_with("/I") || arg.starts_with("-I"))
                    && !arg[2..].starts_with('/')
                {
                    flags.push(arg.clone());
                }
                // /D<macro> 或 /D <macro>
                else if arg.eq_ignore_ascii_case("/D") || arg.eq_ignore_ascii_case("-D") {
                    flags.push(arg.clone());
                    if i + 1 < args.len() {
                        i += 1;
                        flags.push(args[i].clone());
                    }
                } else if arg.len() > 2
                    && (arg.starts_with("/D") || arg.starts_with("-D"))
                    && !arg[2..].starts_with('/')
                {
                    flags.push(arg.clone());
                }
                // /U<macro>
                else if arg.len() > 2
                    && (arg.starts_with("/U") || arg.starts_with("-U"))
                {
                    flags.push(arg.clone());
                }
                // /std:c++XX
                else if arg.starts_with("/std:") || arg.starts_with("-std:") || arg.starts_with("/std") || arg.starts_with("-std=") {
                    flags.push(arg.clone());
                }
                // /FI <file> (force include)
                else if arg.eq_ignore_ascii_case("/FI") || arg.eq_ignore_ascii_case("-FI") {
                    flags.push(arg.clone());
                    if i + 1 < args.len() {
                        i += 1;
                        flags.push(args[i].clone());
                    }
                }
            }
            CompilerStyle::Gnu => {
                // -I<dir> 或 -I <dir>
                if arg == "-I" {
                    flags.push(arg.clone());
                    if i + 1 < args.len() {
                        i += 1;
                        flags.push(args[i].clone());
                    }
                } else if arg.starts_with("-I") {
                    flags.push(arg.clone());
                }
                // -D<macro>
                else if arg == "-D" {
                    flags.push(arg.clone());
                    if i + 1 < args.len() {
                        i += 1;
                        flags.push(args[i].clone());
                    }
                } else if arg.starts_with("-D") {
                    flags.push(arg.clone());
                }
                // -U<macro>
                else if arg == "-U" || (arg.starts_with("-U") && arg.len() > 2) {
                    flags.push(arg.clone());
                    if arg == "-U" && i + 1 < args.len() {
                        i += 1;
                        flags.push(args[i].clone());
                    }
                }
                // -std=c++XX
                else if arg.starts_with("-std=") {
                    flags.push(arg.clone());
                }
                // -include / -isystem / -iquote <file>
                else if arg == "-include" || arg == "-isystem" || arg == "-iquote" {
                    flags.push(arg.clone());
                    if i + 1 < args.len() {
                        i += 1;
                        flags.push(args[i].clone());
                    }
                }
                // -fshort-enums
                else if arg == "-fshort-enums" {
                    flags.push(arg.clone());
                }
            }
        }
        i += 1;
    }
    flags
}

/// 在 PATH 中查找真实编译器，跳过 `skip_dir`。
fn find_real_compiler(compiler_name: &str, skip_dir: &Path) -> Option<PathBuf> {
    let path_var = env::var("PATH").unwrap_or_default();
    let skip_canonical = canonical(skip_dir);

    // 在 PATH 目录中按顺序查找（跳过 skip_dir）
    for dir in env::split_paths(&path_var) {
        if canonical(&dir) == skip_canonical {
            debug(&format!("find_real_compiler: skip dir {}", dir.display()));
            continue;
        }
        // 尝试 <dir>/<name>（带 .exe）
        let candidate = dir.join(compiler_name);
        if candidate.is_file() {
            debug(&format!("find_real_compiler: found {}", candidate.display()));
            return Some(candidate);
        }
        // 若 compiler_name 不含 .exe，也尝试带 .exe 的版本
        if !compiler_name.to_ascii_lowercase().ends_with(".exe") {
            let candidate_exe = dir.join(format!("{}.exe", compiler_name));
            if candidate_exe.is_file() {
                debug(&format!(
                    "find_real_compiler: found {}",
                    candidate_exe.display()
                ));
                return Some(candidate_exe);
            }
        }
    }
    None
}

/// 运行预处理，将结果保存为 .cpp2rust 文件。
/// 返回 true 表示成功（或预处理失败但不致命），false 表示需要中止。
fn preprocess_file(
    real_compiler: &Path,
    cpp_file: &Path,
    preprocess_flags: &[String],
    project_root: &Path,
    feature_root: &Path,
    style: CompilerStyle,
) {
    // 规范化 cpp_file 路径
    let abs_cpp = match cpp_file.canonicalize() {
        Ok(p) => strip_unc_prefix(&p),
        Err(e) => {
            debug(&format!(
                "preprocess_file: canonicalize {} failed: {}",
                cpp_file.display(),
                e
            ));
            return;
        }
    };

    // strip_prefix: abs_cpp 必须位于 project_root 之下
    let rel = match abs_cpp.strip_prefix(project_root) {
        Ok(r) => r,
        Err(_) => {
            debug(&format!(
                "preprocess_file: {} not under project root {}",
                abs_cpp.display(),
                project_root.display()
            ));
            return;
        }
    };

    // 构建输出路径：<feature_root>/c/<rel>/<filename>.cpp2rust
    let out_path = {
        let file_name = abs_cpp
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("unknown");
        let parent = feature_root.join("c").join(rel.parent().unwrap_or(Path::new("")));
        parent.join(format!("{}.cpp2rust", file_name))
    };

    // 创建输出目录
    if let Some(parent) = out_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            debug(&format!(
                "preprocess_file: create_dir_all {} failed: {}",
                parent.display(),
                e
            ));
            return;
        }
    }

    // 保存 .opts 文件（编译选项）
    let opts_path = format!("{}.opts", out_path.display());
    let opts_content: Vec<String> = preprocess_flags
        .iter()
        .map(|f| format!("\"{}\" ", f))
        .collect();
    let _ = fs::write(&opts_path, opts_content.join(""));

    debug(&format!(
        "preprocess_file: {} -> {}",
        abs_cpp.display(),
        out_path.display()
    ));

    // 构建预处理命令
    let mut cmd = Command::new(real_compiler);

    match style {
        CompilerStyle::Msvc => {
            // cl.exe /P /C /Fi<output> <flags> <source>
            // /P: 输出预处理结果到文件
            // 注意：不使用 /EP，以保留行号标记（#line），使 libclang 能正确识别系统头文件。
            // /C: 保留注释
            // /Fi<output>: 指定输出文件路径
            cmd.arg("/P");
            cmd.arg("/C");
            cmd.arg(format!("/Fi{}", out_path.display()));
            cmd.args(preprocess_flags);
            cmd.arg(abs_cpp.as_os_str());
        }
        CompilerStyle::Gnu => {
            // g++.exe -E -C <source> -o <output> <flags>
            cmd.arg("-E");
            cmd.arg("-C");
            cmd.arg(abs_cpp.as_os_str());
            cmd.arg("-o");
            cmd.arg(out_path.as_os_str());
            cmd.args(preprocess_flags);
        }
    }

    // 设置 CPP2RUST_CC_SKIP 防止递归（虽然已用绝对路径，仍作双重保护）
    cmd.env("CPP2RUST_CC_SKIP", "1");

    match cmd.status() {
        Ok(s) if s.success() => {
            debug(&format!(
                "preprocess_file: OK ({})",
                out_path.display()
            ));
        }
        Ok(s) => {
            debug(&format!(
                "preprocess_file: failed for {} exit={}",
                abs_cpp.display(),
                s.code().unwrap_or(-1)
            ));
        }
        Err(e) => {
            debug(&format!(
                "preprocess_file: spawn failed for {}: {}",
                abs_cpp.display(),
                e
            ));
        }
    }
}

fn run() -> i32 {
    // ── 步骤 1：确定我们被当作哪个编译器调用 ──
    let my_exe = match env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[hook-wrapper] current_exe() failed: {}", e);
            return 1;
        }
    };
    let my_dir = my_exe.parent().unwrap_or(Path::new("")).to_path_buf();
    let my_dir_canonical = canonical(&my_dir);

    // argv[0] 在 Windows 上是调用时使用的文件名（包含 .exe）
    let args: Vec<String> = env::args().collect();
    let called_as = if args.is_empty() {
        my_exe
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("hook-wrapper")
            .to_string()
    } else {
        // args[0] 是调用路径，取文件名部分
        Path::new(&args[0])
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("hook-wrapper")
            .to_string()
    };

    debug(&format!(
        "hook-wrapper invoked as '{}' (my_dir={})",
        called_as,
        my_dir.display()
    ));

    // ── 步骤 2：查找真实编译器 ──
    let real_compiler = match find_real_compiler(&called_as, &my_dir_canonical) {
        Some(p) => p,
        None => {
            eprintln!(
                "[hook-wrapper] cannot find real '{}' in PATH (after skipping {})",
                called_as,
                my_dir.display()
            );
            return 1;
        }
    };
    debug(&format!(
        "hook-wrapper real compiler: {}",
        real_compiler.display()
    ));

    let cmd_args: Vec<String> = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        vec![]
    };

    // ── 步骤 3：若 CPP2RUST_CC_SKIP 已设或环境变量缺失，直接透传 ──
    let should_intercept = env::var_os("CPP2RUST_CC_SKIP").is_none()
        && env::var_os("CPP2RUST_PROJECT_ROOT").is_some()
        && env::var_os("CPP2RUST_FEATURE_ROOT").is_some()
        && is_known_compiler(&called_as);

    if should_intercept {
        let style = compiler_style(&called_as);

        // 获取项目根目录和 feature 根目录
        let project_root_raw = env::var("CPP2RUST_PROJECT_ROOT").unwrap();
        let feature_root_raw = env::var("CPP2RUST_FEATURE_ROOT").unwrap();

        let project_root = canonical(Path::new(&project_root_raw));
        let feature_root = canonical(Path::new(&feature_root_raw));

        debug(&format!(
            "hook-wrapper: style={:?} project_root={} feature_root={}",
            style,
            project_root.display(),
            feature_root.display()
        ));

        // 提取预处理 flags 和 C++ 源文件
        let preprocess_flags = extract_preprocess_flags(&cmd_args, style);
        let cpp_files: Vec<PathBuf> = cmd_args
            .iter()
            .filter(|a| arg_is_cppfile(a, style))
            .map(|a| PathBuf::from(a))
            .collect();

        debug(&format!(
            "hook-wrapper: {} cpp file(s), {} flag(s)",
            cpp_files.len(),
            preprocess_flags.len()
        ));

        // 对每个 C++ 文件运行预处理
        for cpp_file in &cpp_files {
            preprocess_file(
                &real_compiler,
                cpp_file,
                &preprocess_flags,
                &project_root,
                &feature_root,
                style,
            );
        }
    } else {
        debug(&format!(
            "hook-wrapper: skipping interception (cc_skip={}, project_root={}, feature_root={}, known={})",
            env::var_os("CPP2RUST_CC_SKIP").is_some(),
            env::var_os("CPP2RUST_PROJECT_ROOT").is_some(),
            env::var_os("CPP2RUST_FEATURE_ROOT").is_some(),
            is_known_compiler(&called_as)
        ));
    }

    // ── 步骤 4：透传给真实编译器 ──
    let mut real_cmd = Command::new(&real_compiler);
    real_cmd.args(&cmd_args);
    // 传递 CPP2RUST_CC_SKIP=1 以防止递归（若 real_compiler 因某种原因再次触发 hook）
    real_cmd.env("CPP2RUST_CC_SKIP", "1");

    match real_cmd.status() {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!(
                "[hook-wrapper] failed to spawn '{}': {}",
                real_compiler.display(),
                e
            );
            1
        }
    }
}

fn main() -> ExitCode {
    let code = run();
    ExitCode::from(code.clamp(0, 255) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_known_compiler() {
        assert!(is_known_compiler("cl.exe"));
        assert!(is_known_compiler("cl"));
        assert!(is_known_compiler("CL.EXE"));
        assert!(is_known_compiler("clang-cl.exe"));
        assert!(is_known_compiler("g++.exe"));
        assert!(is_known_compiler("g++"));
        assert!(is_known_compiler("clang++.exe"));
        assert!(is_known_compiler("c++.exe"));
        assert!(!is_known_compiler("make.exe"));
        assert!(!is_known_compiler("link.exe"));
        assert!(!is_known_compiler("msbuild.exe"));
    }

    #[test]
    fn test_is_cpp_file() {
        assert!(is_cpp_file("foo.cpp"));
        assert!(is_cpp_file("foo.cc"));
        assert!(is_cpp_file("foo.cxx"));
        assert!(is_cpp_file("foo.CPP"));
        assert!(is_cpp_file(r"C:\Users\foo\bar.cpp"));
        assert!(!is_cpp_file("foo.c"));
        assert!(!is_cpp_file("foo.h"));
        assert!(!is_cpp_file("foo.obj"));
        assert!(!is_cpp_file("foo.lib"));
    }

    #[test]
    fn test_compiler_style() {
        assert_eq!(compiler_style("cl.exe"), CompilerStyle::Msvc);
        assert_eq!(compiler_style("clang-cl.exe"), CompilerStyle::Msvc);
        assert_eq!(compiler_style("cl"), CompilerStyle::Msvc);
        assert_eq!(compiler_style("g++.exe"), CompilerStyle::Gnu);
        assert_eq!(compiler_style("clang++.exe"), CompilerStyle::Gnu);
    }

    #[test]
    fn test_extract_preprocess_flags_gnu() {
        let args: Vec<String> = vec![
            "-I/usr/include",
            "-DFOO=1",
            "-DBAR",
            "-std=c++17",
            "-o",
            "foo.o",
            "foo.cpp",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let flags = extract_preprocess_flags(&args, CompilerStyle::Gnu);
        assert!(flags.contains(&"-I/usr/include".to_string()));
        assert!(flags.contains(&"-DFOO=1".to_string()));
        assert!(flags.contains(&"-DBAR".to_string()));
        assert!(flags.contains(&"-std=c++17".to_string()));
        // -o and output file should NOT be in flags
        assert!(!flags.contains(&"-o".to_string()));
        assert!(!flags.contains(&"foo.o".to_string()));
    }

    #[test]
    fn test_extract_preprocess_flags_msvc() {
        let args: Vec<String> = vec![
            "/Iinclude",
            "/DFOO=1",
            "/std:c++17",
            "/Fe:foo.exe",
            "foo.cpp",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let flags = extract_preprocess_flags(&args, CompilerStyle::Msvc);
        assert!(flags.contains(&"/Iinclude".to_string()));
        assert!(flags.contains(&"/DFOO=1".to_string()));
        assert!(flags.contains(&"/std:c++17".to_string()));
        // /Fe should NOT be in flags
        assert!(!flags.iter().any(|f| f.starts_with("/Fe")));
    }

    #[test]
    fn test_arg_is_cppfile_gnu() {
        // 不能直接测试磁盘文件，仅测试不存在文件时返回 false
        assert!(!arg_is_cppfile("-Ifoo", CompilerStyle::Gnu));
        assert!(!arg_is_cppfile("-o", CompilerStyle::Gnu));
        // 源文件路径需要存在于磁盘，这里测试扩展名过滤
        assert!(!arg_is_cppfile("foo.h", CompilerStyle::Gnu));
        assert!(!arg_is_cppfile("foo.obj", CompilerStyle::Gnu));
    }

    #[test]
    fn test_arg_is_cppfile_msvc() {
        assert!(!arg_is_cppfile("/Iinclude", CompilerStyle::Msvc));
        assert!(!arg_is_cppfile("/DFOO", CompilerStyle::Msvc));
        assert!(!arg_is_cppfile("foo.h", CompilerStyle::Msvc));
        assert!(!arg_is_cppfile("foo.obj", CompilerStyle::Msvc));
    }

    #[test]
    fn test_strip_unc_prefix() {
        let p = Path::new(r"\\?\C:\Users\foo");
        let stripped = strip_unc_prefix(p);
        assert_eq!(stripped, PathBuf::from(r"C:\Users\foo"));

        let p2 = Path::new(r"C:\Users\foo");
        assert_eq!(strip_unc_prefix(p2), PathBuf::from(r"C:\Users\foo"));
    }
}
