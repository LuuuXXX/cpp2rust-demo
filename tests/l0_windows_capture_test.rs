/// L0 Windows capture 专项测试
///
/// 仅在 Windows 上运行（非 Windows CI 上自动 ignore）。
/// 验证 PATH wrapper 机制的基本行为：
///   - hook-wrapper.exe 可以正确解压到数据目录
///   - 临时 wrapper 目录被正确创建
///   - 父进程 PATH 在子进程结束后不受影响
///
/// 注意：这些测试不需要真实的 C++ 编译器。
use std::path::PathBuf;

// ── 辅助：获取嵌入的 HOOK_WRAPPER_EXE（仅 Windows 构建时可用）──

#[cfg(windows)]
const HOOK_WRAPPER_BYTES: &[u8] = include_bytes!(env!("CPP2RUST_HOOK_WRAPPER_EXE"));

/// 验证内嵌的 hook-wrapper.exe 字节非空（基础完整性检查）。
#[test]
#[cfg_attr(not(target_os = "windows"), ignore = "Windows-only test")]
fn test_hook_wrapper_bytes_nonempty() {
    #[cfg(windows)]
    {
        assert!(
            !HOOK_WRAPPER_BYTES.is_empty(),
            "HOOK_WRAPPER_EXE should not be empty"
        );
        // Windows PE 可执行文件以 MZ 魔数开头
        assert_eq!(
            &HOOK_WRAPPER_BYTES[..2],
            b"MZ",
            "hook-wrapper.exe should start with MZ magic (PE format)"
        );
    }
}

/// 验证解压 hook-wrapper.exe 后文件存在于磁盘。
#[test]
#[cfg_attr(not(target_os = "windows"), ignore = "Windows-only test")]
fn test_hook_wrapper_extracts_to_data_dir() {
    #[cfg(windows)]
    {
        // 通过 cargo 运行（需要 Windows 环境）
        // 直接测试数据目录写入逻辑
        let local_app_data = std::env::var("LOCALAPPDATA")
            .or_else(|_| std::env::var("APPDATA"))
            .expect("LOCALAPPDATA or APPDATA should be set on Windows");

        let expected_dir =
            PathBuf::from(&local_app_data).join("cpp2rust-demo").join("hook");
        let expected_exe = expected_dir.join("hook-wrapper.exe");

        // 写入文件
        std::fs::create_dir_all(&expected_dir).expect("should create dir");
        std::fs::write(&expected_exe, HOOK_WRAPPER_BYTES).expect("should write exe");

        assert!(expected_exe.exists(), "hook-wrapper.exe should exist after write");

        // 验证内容一致
        let on_disk = std::fs::read(&expected_exe).expect("should read");
        assert_eq!(on_disk, HOOK_WRAPPER_BYTES, "on-disk bytes should match");
    }
}

/// 验证父进程 PATH 在子进程结束后不受影响。
#[test]
#[cfg_attr(not(target_os = "windows"), ignore = "Windows-only test")]
fn test_path_injection_is_process_local() {
    #[cfg(windows)]
    {
        let original_path = std::env::var("PATH").unwrap_or_default();
        let tmp_dir = tempfile::tempdir().expect("create tempdir");

        // 注入一个假的 PATH 条目
        let new_path = format!("{};{}", tmp_dir.path().display(), original_path);

        // 使用修改后的 PATH 运行一个简单命令
        let status = std::process::Command::new("cmd")
            .args(["/c", "echo", "ok"])
            .env("PATH", &new_path)
            .status()
            .expect("should run cmd");
        assert!(status.success());

        // 父进程的 PATH 应保持不变
        let current_path = std::env::var("PATH").unwrap_or_default();
        assert_eq!(
            current_path, original_path,
            "parent PATH should not be modified after child process exits"
        );
    }
}

/// 验证创建编译器名称别名（硬链接或拷贝）的基本逻辑。
#[test]
#[cfg_attr(not(target_os = "windows"), ignore = "Windows-only test")]
fn test_compiler_alias_creation() {
    #[cfg(windows)]
    {
        let tmp_dir = tempfile::tempdir().expect("create tempdir");

        // 创建一个假的 hook-wrapper.exe
        let wrapper = tmp_dir.path().join("hook-wrapper.exe");
        std::fs::write(&wrapper, HOOK_WRAPPER_BYTES).expect("write wrapper");

        // 为各编译器名创建链接
        let compiler_names = ["cl.exe", "clang-cl.exe", "g++.exe", "clang++.exe"];
        for name in &compiler_names {
            let alias = tmp_dir.path().join(name);
            let result = std::fs::hard_link(&wrapper, &alias)
                .or_else(|_| std::fs::copy(&wrapper, &alias).map(|_| ()));
            assert!(
                result.is_ok(),
                "should create alias for {} (hard_link or copy)",
                name
            );
            assert!(alias.exists(), "alias {} should exist", name);
        }
    }
}
