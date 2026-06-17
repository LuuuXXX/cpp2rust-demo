//! C3：脚本可发现性测试
//!
//! 断言 `usage/verify-<lib>-ffi.sh` 本地验证脚本集合与 E2E 集成测试（
//! `.github/workflows/e2e-<lib>.yml` 工作流）的库集合保持一一对应，防止未来新增
//! 真实库时漏配本地验证脚本（或漏配 CI 工作流）。
//!
//! 该测试不依赖 libclang / 子模块，纯文件名集合比对，默认随 `cargo test` 运行。

use std::collections::BTreeSet;
use std::path::Path;

/// 从某目录收集匹配 `<prefix><lib><suffix>` 的文件名，提取中间的 `<lib>` 标识。
fn collect_libs(dir: &str, prefix: &str, suffix: &str) -> BTreeSet<String> {
    let mut libs = BTreeSet::new();
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(dir);
    let entries = std::fs::read_dir(&path)
        .unwrap_or_else(|e| panic!("无法读取目录 {}: {e}", path.display()));
    for entry in entries {
        let entry = entry.expect("读取目录项失败");
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some(rest) = name.strip_prefix(prefix) {
            if let Some(lib) = rest.strip_suffix(suffix) {
                libs.insert(lib.to_string());
            }
        }
    }
    libs
}

#[test]
fn verify_scripts_match_e2e_workflows() {
    // 本地验证脚本：usage/verify-<lib>-ffi.sh
    let scripts = collect_libs("usage", "verify-", "-ffi.sh");
    // E2E 集成测试 CI 工作流：.github/workflows/e2e-<lib>.yml
    let workflows = collect_libs(".github/workflows", "e2e-", ".yml");

    assert!(
        !scripts.is_empty(),
        "未发现任何 usage/verify-*-ffi.sh 脚本，请检查测试路径"
    );
    assert!(
        !workflows.is_empty(),
        "未发现任何 .github/workflows/e2e-*.yml 工作流，请检查测试路径"
    );

    let only_scripts: Vec<_> = scripts.difference(&workflows).cloned().collect();
    let only_workflows: Vec<_> = workflows.difference(&scripts).cloned().collect();

    assert!(
        only_scripts.is_empty() && only_workflows.is_empty(),
        "本地验证脚本与 E2E 工作流的库集合不一致：\n  仅有脚本无 E2E 工作流：{only_scripts:?}\n  仅有 E2E 工作流无脚本：{only_workflows:?}\n  脚本集合：{scripts:?}\n  工作流集合：{workflows:?}\n→ 新增真实库时需同时补 usage/verify-<lib>-ffi.sh 与 .github/workflows/e2e-<lib>.yml"
    );
}

#[test]
fn every_verify_script_sources_common_lib() {
    // 每份 per-library 脚本都应 source 共享库 verify-common.sh（rapidjson 为独立脚本，豁免）。
    let usage = Path::new(env!("CARGO_MANIFEST_DIR")).join("usage");
    let common = usage.join("lib").join("verify-common.sh");
    assert!(
        common.is_file(),
        "缺少共享库 usage/lib/verify-common.sh: {}",
        common.display()
    );

    for entry in std::fs::read_dir(&usage).expect("无法读取 usage 目录") {
        let entry = entry.expect("读取目录项失败");
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "verify-rapidjson-ffi.sh" {
            continue; // rapidjson 为历史独立脚本，不强制复用共享库
        }
        let Some(_) = name
            .strip_prefix("verify-")
            .and_then(|r| r.strip_suffix("-ffi.sh"))
        else {
            continue;
        };
        let body = std::fs::read_to_string(entry.path())
            .unwrap_or_else(|e| panic!("无法读取 {name}: {e}"));
        assert!(
            body.contains("lib/verify-common.sh"),
            "{name} 未 source 共享库 usage/lib/verify-common.sh"
        );
        assert!(
            body.contains("vc_run"),
            "{name} 未调用 vc_run（七阶段骨架入口）"
        );
    }
}
