//! 行号标记扫描（用于 is_from_current_file 判断）
//!
//! 通过解析 `g++ -E` 生成的预处理文件中的 GCC linemarker，
//! 确定哪些字节区间属于用户代码、哪些来自 `.cpp` 文件本身。

/// 判断 clang 实体的起始偏移量是否落在 `cpp_ranges` 范围内，
/// 即实体是否来自当前 `.cpp` 文件本身（而非 `#include` 引入的头文件）。
pub(super) fn entity_is_from_current_file(
    entity: &clang::Entity<'_>,
    cpp_ranges: &[std::ops::Range<u32>],
) -> bool {
    entity
        .get_range()
        .map(|r| {
            let offset = r.get_start().get_file_location().offset;
            cpp_ranges.iter().any(|range| range.contains(&offset))
        })
        .unwrap_or(false)
}

/// 扫描 `g++ -E` 生成的预处理文件内容，返回属于 `.cpp`/`.c` 文件（而非 `.h`/`.hpp` 头文件）
/// 内容的字节偏移量区间列表。
///
/// 原理：预处理文件中包含行号标记（linemarker），格式为
/// `# <行号> "<文件路径>" [标志]`，通过解析这些标记即可知道每段内容来自哪个原始文件。
/// 后缀为 `.h`/`.hpp` 的标记表示进入了头文件，后缀为 `.cpp`/`.c` 的标记表示回到了
/// 主 shim 文件；系统虚拟路径（`<built-in>`、`<command-line>` 等）则跳过。
pub fn cpp_byte_ranges(content: &str) -> Vec<std::ops::Range<u32>> {
    let mut ranges: Vec<std::ops::Range<u32>> = Vec::new();
    let mut in_cpp = false;
    let mut section_start: u32 = 0;
    let mut byte_pos: u32 = 0;

    for line in content.split('\n') {
        let line_byte_len = line.len() as u32 + 1; // +1 表示 '\n'

        let trimmed = line.trim_start();
        if trimmed.starts_with("# ") {
            if let Some((file_path, _flags)) = parse_line_marker(trimmed) {
                // 过滤系统虚拟路径（<built-in>、<command-line> 等）
                let is_virtual = file_path.starts_with('<') || file_path.is_empty();
                // 头文件后缀
                let is_header = file_path.ends_with(".h")
                    || file_path.ends_with(".hpp")
                    || file_path.ends_with(".hh");
                // .cpp/.c 文件（即 shim cpp 自身）
                let is_cpp = !is_virtual && !is_header;

                match (in_cpp, is_cpp) {
                    (true, false) => {
                        // 离开 cpp 区间
                        ranges.push(section_start..byte_pos);
                        in_cpp = false;
                    }
                    (false, true) => {
                        // 进入 cpp 区间（行号标记行本身不算内容，从下一行开始）
                        in_cpp = true;
                        section_start = byte_pos + line_byte_len;
                    }
                    _ => {}
                }
            }
        }

        byte_pos += line_byte_len;
    }

    if in_cpp && section_start < byte_pos {
        let content_end = content.len() as u32;
        ranges.push(section_start..content_end);
    }

    ranges
}

/// 格式：`# <数字> "<路径>" [标志...]`
/// 返回 `(路径, 标志列表)`，其中常见标志含义：
///   1 = 进入新文件，2 = 返回调用文件，3 = 系统头文件，4 = 隐式 extern "C"
fn parse_line_marker(line: &str) -> Option<(&str, Vec<u32>)> {
    // 跳过 "# " 前缀
    let rest = line[2..].trim_start();
    // 跳过数字
    let after_num = rest
        .trim_start_matches(|c: char| c.is_ascii_digit())
        .trim_start();
    // 必须以 '"' 开头
    if !after_num.starts_with('"') {
        return None;
    }
    let inner = &after_num[1..];
    let end = inner.find('"')?;
    let path = &inner[..end];
    // 解析路径后的标志（可选）
    let flags_str = inner[end + 1..].trim();
    let flags: Vec<u32> = flags_str
        .split_whitespace()
        .filter_map(|s| s.parse::<u32>().ok())
        .collect();
    Some((path, flags))
}

/// 扫描预处理文件，返回属于**用户代码**（非系统头）的原始文件路径集合。
///
/// 通过解析行号标记（linemarker），提取不含 flag-3（系统头）且非虚拟路径的文件路径。
/// 返回的路径已规范化（反斜杠→正斜杠，统一小写），以支持跨平台比较。
///
/// **路径规范化**：linemarker 中的路径使用 C-string 转义，Windows 路径如
/// `cpp\\file.cpp`（两个反斜杠）会被直接写入文件。因此需要两步转换：
/// 1. `\\` → `\`（解码 C-string 转义，例如 `C:\\dir\\file.cpp` → `C:\dir\file.cpp`）
/// 2. `\` → `/`（统一分隔符，例如 `C:\dir\file.cpp` → `C:/dir/file.cpp`）
///
/// 若跳过第一步直接替换，则 `cpp\\hello.cpp` 会变成 `cpp//hello.cpp`（双斜杠），
///    导致与 libclang `get_presumed_location()` 返回的路径无法匹配。
///
/// 用法：配合 `entity_presumed_from_user_file` 通过 `get_presumed_location()` 检查
/// 实体是否来自用户代码，该方法不依赖字节偏移量，对 CRLF/路径差异更健壮。
pub fn user_file_paths_from_content(content: &str) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    for line in content.split('\n') {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("# ") {
            continue;
        }
        if let Some((path, flags)) = parse_line_marker(trimmed) {
            let is_virtual = path.starts_with('<') || path.is_empty();
            if !is_virtual && !flags.contains(&3) {
                // 先解码 C-string 中的 \\ 转义（两个反斜杠→一个反斜杠），
                // 再将剩余反斜杠统一为正斜杠，避免路径出现双斜杠（如 cpp//file.cpp）。
                let decoded = path.replace("\\\\", "\\");
                set.insert(decoded.replace('\\', "/").to_lowercase());
            }
        }
    }
    set
}

/// 通过 `get_presumed_location()` 判断实体是否来自用户代码文件。
///
/// `get_presumed_location()` 跟随预处理文件中的 `#line` 指令，返回实体在**原始源文件**
/// 中的路径（而非预处理文件本身）。与字节范围检查互补：当字节偏移量因平台差异（CRLF、
/// 绝对路径、不同预处理器）导致比较失败时，基于路径的判断仍然正确。
///
/// 匹配策略：
///   1. 直接匹配（规范化后完整路径相等）
///   2. 后缀匹配（一方是另一方的后缀，处理绝对路径 vs 相对路径的差异）
pub(super) fn entity_presumed_from_user_file(
    entity: &clang::Entity<'_>,
    user_files: &std::collections::HashSet<String>,
) -> bool {
    if user_files.is_empty() {
        return false; // 无信息时保守处理，由其他检查决定
    }
    let Some(range) = entity.get_range() else {
        return false;
    };
    let (presumed_path, _, _) = range.get_start().get_presumed_location();
    if presumed_path.starts_with('<') || presumed_path.is_empty() {
        return false; // 虚拟路径 = 非用户文件
    }
    let normalized = presumed_path.replace('\\', "/").to_lowercase();
    // 直接匹配
    if user_files.contains(&normalized) {
        return true;
    }
    // 后缀匹配：处理绝对路径（linemarker）vs 相对路径（presumed）或反之
    user_files
        .iter()
        .any(|uf| normalized.ends_with(uf.as_str()) || uf.ends_with(normalized.as_str()))
}

/// 扫描 `g++ -E` 生成的预处理文件，返回属于**本地项目文件**的字节区间。
///
/// "本地项目文件"的判断准则：文件路径以 `main_dir/`（主 `.cpp` 所在目录）或
/// `parent_dir/`（其上一级目录）开头，且不含 flag-3（系统头）。
///
/// 这样既能包含同目录的用户头文件（`examples/*/cpp/*.h`、`references/tinyxml2/*.h`），
/// 也能覆盖"源码与头文件分处不同子目录"的情形（如 fmtlib 的 `src/` 与 `include/`
/// 同属 `references/fmtlib/`）；同时能排除主 `.cpp` 所在项目树之外的三方库头
/// （如 `references/magic_enum/include/` 对于临时驱动文件 `.cpp2rust/.../tmpXXX.cpp`）。
///
/// **降级行为**：若文件不含任何行号标记，则返回覆盖全文的单一区间（与
/// `user_content_byte_ranges` 一致）。
pub fn local_project_byte_ranges(content: &str) -> Vec<std::ops::Range<u32>> {
    // 1. 找到主 .cpp 文件路径（第一个非虚拟、非头文件的 linemarker）
    let main_cpp: Option<String> = {
        let mut found = None;
        for line in content.split('\n') {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("# ") {
                continue;
            }
            if let Some((path, _)) = parse_line_marker(trimmed) {
                let is_virtual = path.starts_with('<') || path.is_empty();
                if is_virtual {
                    continue;
                }
                let is_header = path.ends_with(".h")
                    || path.ends_with(".hpp")
                    || path.ends_with(".hh");
                if !is_header {
                    found = Some(path.replace("\\\\", "\\").replace('\\', "/"));
                    break;
                }
            }
        }
        found
    };

    let Some(main_cpp_path) = main_cpp else {
        // 无行号标记 → 覆盖全文
        #[allow(clippy::single_range_in_vec_init)]
        return vec![0..content.len() as u32];
    };

    // 2. 计算 main_dir 和 parent_dir（均以 '/' 结尾，便于前缀匹配）
    let main_dir = {
        let p = std::path::Path::new(&main_cpp_path);
        match p.parent() {
            Some(d) => {
                let s = d.to_string_lossy().replace('\\', "/");
                if s.is_empty() {
                    ".".to_string()
                } else {
                    s
                }
            }
            None => ".".to_string(),
        }
    };
    let main_dir_prefix = if main_dir.ends_with('/') {
        main_dir.clone()
    } else {
        format!("{}/", main_dir)
    };
    let parent_dir_prefix: String = {
        let p = std::path::Path::new(&main_dir);
        match p.parent() {
            Some(d) => {
                let s = d.to_string_lossy().replace('\\', "/");
                if s.len() > 1 {
                    // 有意义的父目录（不是 "/" 或 "."）
                    if s.ends_with('/') {
                        s
                    } else {
                        format!("{}/", s)
                    }
                } else {
                    String::new() // 父目录过短，不使用
                }
            }
            None => String::new(),
        }
    };

    // 判断给定路径是否属于本地项目
    let is_local = |path: &str| -> bool {
        let normalized = path.replace("\\\\", "\\").replace('\\', "/");
        if normalized.starts_with(&main_dir_prefix) {
            return true;
        }
        if !parent_dir_prefix.is_empty() && normalized.starts_with(&parent_dir_prefix) {
            return true;
        }
        false
    };

    // 3. 扫描 linemarker，构建本地项目字节区间
    let mut ranges: Vec<std::ops::Range<u32>> = Vec::new();
    let mut in_local = false;
    let mut section_start: u32 = 0;
    let mut byte_pos: u32 = 0;
    let mut found_any_marker = false;

    for line in content.split('\n') {
        let line_byte_len = line.len() as u32 + 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with("# ") {
            if let Some((file_path, flags)) = parse_line_marker(trimmed) {
                found_any_marker = true;
                let is_virtual = file_path.starts_with('<') || file_path.is_empty();
                let is_system = flags.contains(&3) || is_virtual;
                let file_is_local = !is_system && is_local(file_path);

                match (in_local, file_is_local) {
                    (true, false) => {
                        ranges.push(section_start..byte_pos);
                        in_local = false;
                    }
                    (false, true) => {
                        in_local = true;
                        section_start = byte_pos + line_byte_len;
                    }
                    _ => {}
                }
            }
        }
        byte_pos += line_byte_len;
    }

    if in_local && section_start < byte_pos {
        ranges.push(section_start..content.len() as u32);
    }

    if !found_any_marker {
        #[allow(clippy::single_range_in_vec_init)]
        return vec![0..content.len() as u32];
    }

    ranges
}

/// 扫描 `g++ -E`（不带 `-P`）生成的预处理文件，返回属于**用户代码**（非系统头）的字节区间。
///
/// GCC linemarker 格式：`# <行号> "<文件路径>" [标志...]`
/// 标志 `3` 表示该段内容来自系统头文件；不含标志 `3` 的区间属于用户代码。
///
/// **降级行为**：若文件不含任何行号标记（例如以 `-P` 生成），则返回覆盖全文的单一区间，
/// 使调用方的字节偏移过滤退化为"全部接受"，不引入额外过滤。
pub fn user_content_byte_ranges(content: &str) -> Vec<std::ops::Range<u32>> {
    let mut ranges: Vec<std::ops::Range<u32>> = Vec::new();
    let mut in_user = false;
    let mut section_start: u32 = 0;
    let mut byte_pos: u32 = 0;
    let mut found_any_marker = false;

    for line in content.split('\n') {
        let line_byte_len = line.len() as u32 + 1; // +1 表示 '\n'

        let trimmed = line.trim_start();
        if trimmed.starts_with("# ") {
            if let Some((file_path, flags)) = parse_line_marker(trimmed) {
                found_any_marker = true;
                let is_virtual = file_path.starts_with('<') || file_path.is_empty();
                // 含标志 3 → 系统头；虚拟路径也视为非用户内容
                let is_system = flags.contains(&3) || is_virtual;
                let is_user = !is_system;

                match (in_user, is_user) {
                    (true, false) => {
                        ranges.push(section_start..byte_pos);
                        in_user = false;
                    }
                    (false, true) => {
                        in_user = true;
                        section_start = byte_pos + line_byte_len;
                    }
                    _ => {}
                }
            }
        }

        byte_pos += line_byte_len;
    }

    if in_user && section_start < byte_pos {
        let content_end = content.len() as u32;
        ranges.push(section_start..content_end);
    }

    // 没有行号标记（如 -P 生成）→ 覆盖全文，使字节过滤退化为无过滤
    if !found_any_marker {
        #[allow(clippy::single_range_in_vec_init)]
        return vec![0..content.len() as u32];
    }

    ranges
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_line_marker ──────────────────────────────────────────────────

    #[test]
    fn test_parse_line_marker_basic() {
        let (path, flags) = parse_line_marker("# 1 \"myfile.cpp\"").unwrap();
        assert_eq!(path, "myfile.cpp");
        assert!(flags.is_empty());
    }

    #[test]
    fn test_parse_line_marker_with_flags() {
        let (path, flags) = parse_line_marker("# 47 \"/usr/include/wchar.h\" 3 4").unwrap();
        assert_eq!(path, "/usr/include/wchar.h");
        assert_eq!(flags, vec![3, 4]);
    }

    #[test]
    fn test_parse_line_marker_system_flag() {
        let (_, flags) = parse_line_marker("# 1 \"/usr/include/stdio.h\" 3").unwrap();
        assert!(flags.contains(&3));
    }

    #[test]
    fn test_parse_line_marker_virtual_path() {
        let (path, _) = parse_line_marker("# 1 \"<built-in>\" 1").unwrap();
        assert_eq!(path, "<built-in>");
    }

    #[test]
    fn test_parse_line_marker_not_a_marker() {
        assert!(parse_line_marker("int foo();").is_none());
        assert!(parse_line_marker("# pragma once").is_none());
    }

    // ── user_content_byte_ranges ───────────────────────────────────────────

    #[test]
    fn test_user_ranges_excludes_system_header() {
        // 简单预处理片段：用户代码 → 系统头（flag 3）→ 返回用户代码
        let content = "# 1 \"myfile.cpp\"\nint a;\n# 1 \"/usr/include/stdio.h\" 3\nvoid sys();\n# 2 \"myfile.cpp\" 2\nint b;\n";
        let ranges = user_content_byte_ranges(content);
        // "int a;\n" 和 "int b;\n" 属于用户代码，"void sys();\n" 不属于
        let user_text: String = ranges
            .iter()
            .map(|r| &content[r.start as usize..r.end as usize])
            .collect();
        assert!(user_text.contains("int a;"), "应包含用户代码 'int a;'");
        assert!(user_text.contains("int b;"), "应包含用户代码 'int b;'");
        assert!(
            !user_text.contains("void sys();"),
            "不应包含系统头函数 'void sys();'"
        );
    }

    #[test]
    fn test_user_ranges_no_markers_fallback() {
        // 无行号标记（-P 生成）→ 返回覆盖全文的单一区间（0..content.len()）
        let content = "int foo();\nvoid bar();\n";
        let ranges = user_content_byte_ranges(content);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start, 0);
        assert_eq!(ranges[0].end, content.len() as u32);
    }

    #[test]
    fn test_user_ranges_virtual_path_excluded() {
        // <built-in> / <command-line> 不属于用户代码
        let content = "# 1 \"<built-in>\"\n__builtin_stuff;\n# 1 \"myfile.cpp\"\nint user;\n";
        let ranges = user_content_byte_ranges(content);
        let user_text: String = ranges
            .iter()
            .map(|r| &content[r.start as usize..r.end as usize])
            .collect();
        assert!(user_text.contains("int user;"));
        assert!(!user_text.contains("__builtin_stuff;"));
    }
}
