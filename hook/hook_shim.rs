// hook_shim.rs — Windows 编译器 Shim，用于替代 Linux/macOS 的 LD_PRELOAD 捕获机制。
//
// 工作原理：
//   1. capture::build_hook_windows() 将本文件内容写入临时目录并用 rustc 编译为 hook_shim.exe
//   2. capture::run_with_hook_windows() 将 hook_shim.exe 复制到另一个临时目录，
//      以真实编译器的基名（如 g++.exe / clang++.exe）命名，然后将该目录插入 PATH 最前面。
//   3. 构建系统调用 g++/clang++ 时实际触发本 shim：
//      a. 将所有参数原样转发给真实编译器（通过 CPP2RUST_REAL_CC 找到）完成正常编译。
//      b. 若参数含 -c（编译模式），额外运行预处理器（-E -C）生成 .cpp2rust 捕获文件。
//
// 环境变量：
//   CPP2RUST_REAL_CC      — 真实编译器的完整路径（如 C:\msys64\mingw64\bin\g++.exe）
//   CPP2RUST_PROJECT_ROOT — 用户 C++ 项目根目录（用于计算相对路径）
//   CPP2RUST_FEATURE_ROOT — .cpp2rust/<feature>/ 输出目录

use std::env;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

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

    // ── 步骤 2：若为编译模式（-c），额外生成 .cpp2rust 捕获文件 ──
    let is_compile_mode = args.iter().any(|a| a == "-c");
    if !is_compile_mode {
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
        // 跳过选项标志及其参数
        if arg.starts_with('-') {
            // 带值的选项（如 -o <file>、-I <dir>）跳过下一个 token
            if matches!(arg.as_str(), "-o" | "-I" | "-isystem" | "-include" | "-MF" | "-MT") {
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        let p = Path::new(arg.as_str());
        if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            if matches!(ext, "cpp" | "cc" | "cxx" | "C") {
                preprocess_file(&real_cc, p, &args, &project_root, &feature_root);
            }
        }
        i += 1;
    }

    exit(0);
}

/// 对单个 C++ 源文件运行预处理器（-E -C），将结果写入 feature_root 下对应的 .cpp2rust 文件。
fn preprocess_file(
    cc: &str,
    src: &Path,
    all_args: &[String],
    project_root: &Path,
    feature_root: &Path,
) {
    // 将输入路径规范化为绝对路径
    let abs_src = match src.canonicalize() {
        Ok(p) => p,
        Err(_) => return,
    };

    // 计算相对于 project_root 的路径
    let rel = match abs_src.strip_prefix(project_root) {
        Ok(r) => r,
        Err(_) => return, // 不在 project_root 下，跳过
    };

    // 输出路径：feature_root/<rel_path>.cpp2rust
    let out_path = feature_root.join(rel).with_extension("cpp2rust");
    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

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
