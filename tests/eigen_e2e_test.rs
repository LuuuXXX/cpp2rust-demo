//! L4-Extended: Eigen 库端到端集成测试
//!
//! 针对系统安装的 Eigen 库（libeigen3-dev）执行 init 阶段转换，
//! 验证工具在密集模板实例化（`ClassTemplateSpecialization`）场景下的鲁棒性：
//! - `Eigen::Matrix3f`（`Eigen::Matrix<float,3,3>`）是具体模板特化类
//! - 工具应能不崩溃地处理 Eigen 的大量内联模板展开
//! - 生成的 cpp! 块应包含必要 include，import_lib! 应包含 extern-C 函数绑定
//!
//! 运行前提：
//! - 已安装 libeigen3-dev：`sudo apt-get install libeigen3-dev`
//!
//! 运行方式：
//! ```bash
//! cargo test --test eigen_e2e_test -- --ignored
//! ```

use cpp2rust_demo::{ast_parser, extractor, generator::hicc_codegen};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const EIGEN_SHIM_DIR: &str = "references/eigen_sys/shim";

const EIGEN_SHIM_SOURCES: &[&str] = &["matrix_ffi.cpp"];

// ─────────────────────────────────────────────────────────────────
//  辅助函数
// ─────────────────────────────────────────────────────────────────

/// 检测 Eigen 系统头文件是否可用。
fn eigen_headers_available() -> bool {
    // libeigen3-dev 标准安装路径
    Path::new("/usr/include/eigen3/Eigen/Dense").exists()
        || Path::new("/usr/local/include/eigen3/Eigen/Dense").exists()
}

/// 获取 Eigen 头文件 include 路径。
fn eigen_include_dirs() -> Vec<String> {
    // 优先 pkg-config
    if let Ok(output) = Command::new("pkg-config")
        .args(["--cflags-only-I", "eigen3"])
        .output()
    {
        if output.status.success() {
            let flags = String::from_utf8_lossy(&output.stdout);
            let dirs: Vec<String> = flags
                .split_whitespace()
                .filter_map(|f| f.strip_prefix("-I"))
                .map(|s| s.to_string())
                .collect();
            if !dirs.is_empty() {
                return dirs;
            }
        }
    }
    // 回退到标准路径
    for path in &["/usr/include/eigen3", "/usr/local/include/eigen3"] {
        if Path::new(path).exists() {
            return vec![path.to_string()];
        }
    }
    vec!["/usr/include/eigen3".to_string()]
}

/// 使用 g++ -E -C 预处理 C++ 文件。
fn preprocess(
    src: &Path,
    include_dirs: &[String],
    out_dir: &Path,
    unit_name: &str,
) -> Option<PathBuf> {
    let out = out_dir.join(format!("{}.cpp2rust", unit_name));
    let mut cmd = Command::new("g++");
    cmd.args(["-E", "-C", "-w", "-std=c++17"]);
    for inc in include_dirs {
        cmd.arg(format!("-I{}", inc));
    }
    cmd.arg(src).arg("-o").arg(&out);
    let ok = cmd.status().map(|s| s.success()).unwrap_or(false);
    if ok { Some(out) } else { None }
}

// ─────────────────────────────────────────────────────────────────
//  L4-Extended-Eigen 测试
// ─────────────────────────────────────────────────────────────────

/// L4-Extended-Eigen：对 eigen_sys/shim/ 中的 extern-C 包装文件执行 init 阶段转换。
///
/// Eigen 是纯模板矩阵库，会触发大量 ClassTemplateSpecialization 实例化。
/// 本测试验证工具能：
/// 1. 在大量模板展开下不崩溃、不 OOM
/// 2. 成功生成 hicc::import_lib! 块（含 extern-C 函数绑定）
/// 3. cpp! 块包含 Eigen 头文件 include
///
/// 测试需要系统安装 libeigen3-dev；使用 `cargo test -- --ignored` 显式运行。
#[test]
#[ignore = "需要 libeigen3-dev；运行方式：cargo test --test eigen_e2e_test -- --ignored"]
fn eigen_matrix_shim_ffi_generates_importlib() {
    if !eigen_headers_available() {
        eprintln!("⚠ 跳过 Eigen E2E 测试：未找到 Eigen 系统头文件（请安装 libeigen3-dev）");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let preprocess_dir = tmp.path().join("c");
    std::fs::create_dir_all(&preprocess_dir).unwrap();

    let shim_dir = PathBuf::from(EIGEN_SHIM_DIR);
    let include_dirs = eigen_include_dirs();

    assert!(
        shim_dir.exists(),
        "shim 目录不存在：{}\n  请确认 references/eigen_sys/shim/ 目录已就绪",
        shim_dir.display()
    );

    let mut processed = 0usize;
    let mut failed: Vec<String> = Vec::new();
    let mut skipped: Vec<&str> = Vec::new();

    for src_name in EIGEN_SHIM_SOURCES {
        let src_path = shim_dir.join(src_name);
        if !src_path.exists() {
            skipped.push(src_name);
            continue;
        }
        let unit_name = src_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unit")
            .to_string();

        let preprocessed =
            match preprocess(&src_path, &include_dirs, &preprocess_dir, &unit_name) {
                Some(p) => p,
                None => {
                    skipped.push(src_name);
                    continue;
                }
            };

        // 检查预处理文件大小（Eigen 展开后可能很大，记录为信息）
        if let Ok(meta) = std::fs::metadata(&preprocessed) {
            println!(
                "  [INFO] {} 预处理文件大小：{} KB",
                unit_name,
                meta.len() / 1024
            );
        }

        let ast = match ast_parser::parse_preprocessed(&preprocessed) {
            Ok(a) => a,
            Err(e) => {
                failed.push(format!("{}: AST 解析失败: {}", unit_name, e));
                continue;
            }
        };

        let (sys_includes, proj_header) = extractor::read_source_includes(&src_path);
        let spec = extractor::extract(&ast, &unit_name, &sys_includes, proj_header.as_deref());
        let code = hicc_codegen::generate(&spec);

        // 验证 1：必须包含 hicc::cpp! 块
        if !code.contains("hicc::cpp! {") {
            failed.push(format!("{}: 生成代码缺少 hicc::cpp! 块", unit_name));
            continue;
        }

        // 验证 2：必须有 import_lib! 块（extern-C 函数绑定）
        if !code.contains("hicc::import_lib! {") {
            failed.push(format!(
                "{}: 生成代码缺少 hicc::import_lib! 块（fn_bindings={}）",
                unit_name,
                spec.lib_spec.fn_bindings.len()
            ));
            continue;
        }

        // 验证 3：函数绑定属性检查
        if !code.contains("#[cpp(func = \"") {
            failed.push(format!(
                "{}: import_lib! 块缺少 #[cpp(func = \"...\")] 注解",
                unit_name
            ));
            continue;
        }

        println!(
            "  [OK] {}: {} fn_bindings, {} class_specs",
            unit_name,
            spec.lib_spec.fn_bindings.len(),
            spec.class_specs.len()
        );
        processed += 1;
    }

    assert!(
        skipped.is_empty(),
        "eigen-shim-ffi: {} 个文件预处理失败:\n{}",
        skipped.len(),
        skipped.join("\n")
    );

    assert!(
        failed.is_empty(),
        "eigen-shim-ffi: {} 个 shim 文件未能生成预期 FFI 绑定:\n{}",
        failed.len(),
        failed.join("\n")
    );

    assert_eq!(
        processed,
        EIGEN_SHIM_SOURCES.len(),
        "eigen-shim-ffi: 期望处理 {} 个文件，实际 {}",
        EIGEN_SHIM_SOURCES.len(),
        processed
    );

    println!("\neigen-shim-ffi: {} 个 shim 文件全部生成 import_lib! FFI 绑定 ✓", processed);
}
