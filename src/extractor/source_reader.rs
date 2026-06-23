//! 源文件 include 读取器（Phase 3 辅助）
//!
//! 从原始 `.cpp` / `.h` / `.hpp` 文件中提取 `#include` 行与 `using namespace` 指令，
//! 供 `extract()` 注入到生成的 `hicc::cpp!` 块顶部，确保 C++ shim 可正确编译。

use std::fs;

/// 读取原始 .cpp 和 .h 文件的 include 行
///
/// 返回 `(system_includes, project_header)`
/// 顺序规则：
///   1. header-only includes（只在头文件中出现、不在 .cpp 中出现）按头文件顺序排前
///   2. cpp includes（.cpp 中出现的系统 include）按 .cpp 文件中出现的顺序排后
///
/// 头文件扩展名按 `.h` → `.hpp` → `.hxx` 顺序探测，取第一个存在的文件，
/// 以便兼容同时使用 `.hpp`（如 rapidjson、Eigen）的项目。
pub fn read_source_includes(cpp_path: &std::path::Path) -> (Vec<String>, Option<String>) {
    let cpp_content = match fs::read_to_string(cpp_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "warning: cpp2rust: failed to read source file '{}': {}",
                cpp_path.display(),
                e
            );
            String::new()
        }
    };

    // 按优先级探测对应头文件（.h → .hpp → .hxx）
    let h_content = ["h", "hpp", "hxx"]
        .iter()
        .map(|ext| cpp_path.with_extension(ext))
        .find_map(|p| fs::read_to_string(&p).ok())
        .unwrap_or_default();

    let mut project: Option<String> = None;

    // 收集头文件中的系统 include（保序）
    let h_includes: Vec<String> = h_content
        .lines()
        .filter_map(|line| {
            let t = line.trim();
            let rest = t.strip_prefix("#include ")?;
            let rest = rest.trim();
            if rest.starts_with('<') {
                Some(format!("#include {}", rest))
            } else {
                None
            }
        })
        .collect();
    // 收集 .cpp 中需要在 cpp! 块顶部重放的前置指令（保序）：
    //   - 系统 include（`<...>`）
    //   - 第三方/跨单元引用的引号 include（`"..."`）；但**首个**引号 include 视为本单元
    //     项目头（project_header），由调用方单独处理，不在此重放
    //   - `using namespace ...;` 指令（使第三方头中未限定的类型在 cpp! 块内可解析）
    // 背景：以 rapidjson shim 为例，`allocator_ffi.cpp` 含 `#include "rapidjson/allocators.h"`
    // 与 `using namespace rapidjson;`。若丢失，内联到 cpp! 块的实现会因 `CrtAllocator` 等
    // 未声明而编译失败。仅保留首个引号 include（旧行为）会漏掉第三方头与跨单元句柄头。
    let mut cpp_includes: Vec<String> = Vec::new();
    let mut cpp_seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for line in cpp_content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("#include ") {
            let rest = rest.trim();
            if rest.starts_with('<') {
                let inc = format!("#include {}", rest);
                if cpp_seen.insert(inc.clone()) {
                    cpp_includes.push(inc);
                }
            } else if rest.starts_with('"') {
                let hdr = rest.trim_matches('"');
                if project.is_none() {
                    project = Some(hdr.to_string());
                } else {
                    // 首个引号 include 之外的引号 include（第三方/跨单元头）需重放
                    let inc = format!("#include \"{}\"", hdr);
                    if cpp_seen.insert(inc.clone()) {
                        cpp_includes.push(inc);
                    }
                }
            }
        } else if t.starts_with("using namespace ") && t.ends_with(';') {
            // 保留命名空间引入，使第三方未限定类型在内联实现中可解析
            if cpp_seen.insert(t.to_string()) {
                cpp_includes.push(t.to_string());
            }
        }
    }
    // 合并：header-only 优先（按头文件顺序），然后 cpp 中的按顺序
    let mut system: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();

    // 1. header-only includes
    for inc in &h_includes {
        if !cpp_seen.contains(inc) && seen.insert(inc.as_str()) {
            system.push(inc.clone());
        }
    }

    // 2. cpp 前置指令（按 cpp 文件顺序，含同时出现在头文件中的 include）
    for inc in &cpp_includes {
        if seen.insert(inc.as_str()) {
            system.push(inc.clone());
        }
    }

    (system, project)
}
