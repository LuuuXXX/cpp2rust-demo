/// 构建脚本：在 Windows 目标上编译 hook-wrapper，并通过 rustc-env 传递产物路径。
///
/// 说明：
///   - 仅当目标操作系统为 Windows 时执行 hook-wrapper 的构建
///   - 产物路径通过 `CPP2RUST_HOOK_WRAPPER_EXE` 环境变量传递给 `capture.rs`，
///     由 `include_bytes!(env!("CPP2RUST_HOOK_WRAPPER_EXE"))` 将 .exe 内嵌进主 binary
///   - 非 Windows 目标时不执行任何操作（hook.cpp 路径依然有效）
use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "windows" {
        build_hook_wrapper();
    }

    // 非 Windows 目标：无需额外操作，hook.cpp 通过 include_str! 直接嵌入。
}

fn build_hook_wrapper() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");

    let wrapper_manifest = Path::new(&manifest_dir).join("hook-wrapper").join("Cargo.toml");
    let wrapper_target_dir = Path::new(&out_dir).join("hook-wrapper-target");

    println!("cargo:rerun-if-changed=hook-wrapper/src/main.rs");
    println!("cargo:rerun-if-changed=hook-wrapper/Cargo.toml");

    // 构建 hook-wrapper（release 模式以减小体积）
    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--manifest-path",
            wrapper_manifest.to_str().expect(
                "hook-wrapper path contains non-UTF-8 characters. \
                 Please place the project in a directory with an ASCII-compatible path."
            ),
            "--target-dir",
            wrapper_target_dir.to_str().expect(
                "OUT_DIR contains non-UTF-8 characters. \
                 Please use a build directory with an ASCII-compatible path."
            ),
        ])
        .status()
        .expect("failed to invoke cargo to build hook-wrapper");

    if !status.success() {
        panic!(
            "hook-wrapper build failed (see output above). \
             Make sure Rust toolchain is available."
        );
    }

    let exe_path = wrapper_target_dir.join("release").join("hook-wrapper.exe");
    if !exe_path.exists() {
        panic!(
            "hook-wrapper.exe not found at {} after successful build",
            exe_path.display()
        );
    }

    // 将 exe 路径通过 rustc-env 传递给 capture.rs 中的 include_bytes! 调用
    println!(
        "cargo:rustc-env=CPP2RUST_HOOK_WRAPPER_EXE={}",
        exe_path.display()
    );
}
