//! 将 `hicc_codegen` 生成的 `.rs` 文件解析为结构化数据，供 merge 命令使用。
//!
//! 解析目标格式：
//! ```rust
//! hicc::cpp! { ... }
//! hicc::import_class! { #[cpp(class = "Foo")] class Foo { ... } }
//! hicc::import_lib!  { #![link_name = "foo"] class Foo; fn foo_new(...); }
//! ```

// ─────────────────────────────────────────────
//  数据结构
// ─────────────────────────────────────────────

/// 一个 unit `.rs` 文件解析后的内容
#[derive(Debug, Default)]
pub struct ParsedUnit {
    /// `hicc::cpp!` 块内的行（不含外层花括号行）
    pub cpp_lines: Vec<String>,
    /// `hicc::cpp!` 块中识别到的完整模板类体（规范化后的字符串，用于跨翻译单元块级去重）。
    /// 每个元素对应一个完整的 `template<...> class/struct { ... };` 块，
    /// 已去掉行尾空白并折叠连续空行。
    pub template_bodies: Vec<String>,
    /// 所有 `hicc::import_class!` 块
    pub class_blocks: Vec<ParsedClassBlock>,
    /// `hicc::import_lib!` 块（最多一个；多个时取最后一个合并结果不变）
    pub lib_block: Option<ParsedLibBlock>,
}

/// 单个 `hicc::import_class!` 块
#[derive(Debug)]
pub struct ParsedClassBlock {
    pub class_name: String,
    /// 完整属性行，如 `#[cpp(class = "Foo")]`、`#[cpp(class = "Foo", destroy = "foo_del")]` 或 `#[interface]`
    pub class_attr: String,
    pub methods: Vec<BlockMethod>,
    /// 若 `class_name` 为模板特化（如 `"Stack<int>"`），此字段存储模板基类名（如 `"Stack"`）。
    /// 普通非模板类为 `None`。
    pub template_base: Option<String>,
}

/// 类方法
#[derive(Debug, Clone)]
pub struct BlockMethod {
    /// `#[cpp(method = "...")]` 属性行（去掉前导空白）
    pub attr: String,
    /// `fn foo(...);` 行（去掉前导空白）
    pub fn_sig: String,
}

/// `hicc::import_lib!` 块
#[derive(Debug)]
pub struct ParsedLibBlock {
    /// `#![link_name = "..."]` 值
    pub link_name: String,
    /// `class Foo;` 等前向声明行
    pub fwd_decls: Vec<String>,
    /// 函数绑定
    pub fn_bindings: Vec<ParsedFnBinding>,
}

/// import_lib! 中的单个函数绑定
#[derive(Debug, Clone)]
pub struct ParsedFnBinding {
    /// `#[cpp(func = "...")]` 属性行
    pub attr: String,
    /// `fn foo(...);` 行（含可能的 `unsafe `）
    pub fn_sig: String,
}

// ─────────────────────────────────────────────
//  解析入口
// ─────────────────────────────────────────────

/// 从 Rust 源文件中提取所有顶层 hicc 宏调用块（`hicc::cpp!`、`hicc::import_class!`、
/// `hicc::import_lib!`）的原始文本，包含开头宏调用行和末尾 `}`。
///
/// 使用与生产代码相同的字符串感知花括号计数逻辑（`count_brace_delta`），
/// 确保测试使用的块边界检测与实际合并器完全一致。
pub fn extract_block_texts(src: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut in_block = false;
    let mut depth: i32 = 0;
    let mut block_lines: Vec<String> = Vec::new();

    for line in src.lines() {
        let trimmed = line.trim();
        if !in_block {
            if detect_block_start(trimmed).is_some() {
                in_block = true;
                depth = 0;
                block_lines.clear();
                block_lines.push(line.to_string());
                depth += count_brace_delta(trimmed);
            }
        } else {
            block_lines.push(line.to_string());
            depth += count_brace_delta(trimmed);
            if depth <= 0 {
                result.push(block_lines.join("\n"));
                in_block = false;
                depth = 0;
            }
        }
    }
    result
}

/// 解析一个 unit `.rs` 文件的内容，提取三类 hicc 块。
pub fn parse_unit_rs(src: &str) -> ParsedUnit {
    let mut unit = ParsedUnit::default();

    // 先将源码切分为顶层 hicc 块
    let raw_blocks = extract_raw_blocks(src);

    for rb in raw_blocks {
        match rb.kind {
            BlockKind::Cpp => {
                unit.cpp_lines = parse_cpp_content(&rb.inner_lines);
                unit.template_bodies = extract_template_bodies(&unit.cpp_lines);
            }
            BlockKind::ImportClass => {
                if let Some(cb) = parse_class_content(&rb.inner_lines) {
                    unit.class_blocks.push(cb);
                }
            }
            BlockKind::ImportLib => {
                if let Some(lb) = parse_lib_content(&rb.inner_lines) {
                    unit.lib_block = Some(lb);
                }
            }
        }
    }

    unit
}

// ─────────────────────────────────────────────
//  顶层块提取
// ─────────────────────────────────────────────

#[derive(Debug, PartialEq)]
enum BlockKind {
    Cpp,
    ImportClass,
    ImportLib,
}

struct RawBlock {
    kind: BlockKind,
    /// 外层花括号内的行（不含 `hicc::xxx! {` 行和末尾 `}` 行）
    inner_lines: Vec<String>,
}

/// 从源文件提取所有顶层 hicc 块（深度 = 1 的内容）
fn extract_raw_blocks(src: &str) -> Vec<RawBlock> {
    let mut blocks: Vec<RawBlock> = Vec::new();
    let mut current_kind: Option<BlockKind> = None;
    let mut inner_lines: Vec<String> = Vec::new();
    let mut depth: i32 = 0;

    for line in src.lines() {
        let trimmed = line.trim();

        if current_kind.is_none() {
            // 寻找块开始
            if let Some(kind) = detect_block_start(trimmed) {
                current_kind = Some(kind);
                inner_lines.clear();
                depth = count_brace_delta(trimmed);
                // 如果整行包含 `{` 且 depth 变为 1，下一行才是 inner content
                continue;
            }
        } else {
            // 在块内
            let delta = count_brace_delta(trimmed);
            depth += delta;

            if depth <= 0 {
                // 这是关闭块的 `}`
                if let Some(kind) = current_kind.take() {
                    blocks.push(RawBlock {
                        kind,
                        // mem::take 将 inner_lines 的所有权移入 RawBlock，避免克隆后再清空
                        inner_lines: std::mem::take(&mut inner_lines),
                    });
                }
                depth = 0;
            } else {
                inner_lines.push(line.to_string());
            }
        }
    }

    blocks
}

fn detect_block_start(trimmed: &str) -> Option<BlockKind> {
    if trimmed.starts_with("hicc::cpp!") {
        Some(BlockKind::Cpp)
    } else if trimmed.starts_with("hicc::import_class!") {
        Some(BlockKind::ImportClass)
    } else if trimmed.starts_with("hicc::import_lib!") {
        Some(BlockKind::ImportLib)
    } else {
        None
    }
}

/// 计算一行中 `{` 和 `}` 的净差值（不考虑字符串/注释内的括号）
fn count_brace_delta(line: &str) -> i32 {
    let mut delta = 0i32;
    let mut in_string = false;
    let mut escape_next = false;
    let mut prev_char = ' ';

    for ch in line.chars() {
        if escape_next {
            escape_next = false;
            prev_char = ch;
            continue;
        }
        if ch == '\\' && in_string {
            escape_next = true;
            prev_char = ch;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
        }
        if !in_string {
            if ch == '{' {
                // 跳过 `macro!{` 形式（`{` 紧接 `!`）中的花括号不计（宏调用块由外层计数）
                if prev_char != '!' {
                    delta += 1;
                }
            } else if ch == '}' {
                delta -= 1;
            }
        }
        prev_char = ch;
    }
    delta
}

// ─────────────────────────────────────────────
//  各块内容解析
// ─────────────────────────────────────────────

/// 解析 `hicc::cpp!` 块内容 → 返回内容行（去掉首尾空行）
fn parse_cpp_content(inner_lines: &[String]) -> Vec<String> {
    let lines: Vec<String> = inner_lines
        .iter()
        .map(|l| {
            // 去掉 4 个空格前缀（codegen 生成时 indent 是 4 空格）
            if let Some(stripped) = l.strip_prefix("    ") {
                stripped.to_string()
            } else {
                l.trim_end().to_string()
            }
        })
        .collect();

    // 去掉首尾空行
    let start = lines
        .iter()
        .position(|l| !l.is_empty())
        .unwrap_or(lines.len());
    let end = lines
        .iter()
        .rposition(|l| !l.is_empty())
        .map(|i| i + 1)
        .unwrap_or(0);
    if start >= end {
        Vec::new()
    } else {
        lines[start..end].to_vec()
    }
}

/// 从 `class Foo {` 或 `class Foo {}` 行中提取类名。
fn extract_class_name_from_line(line: &str) -> String {
    let rest = line.trim_start_matches("class ").trim();
    rest.split(|c: char| !c.is_alphanumeric() && c != '_')
        .next()
        .unwrap_or("")
        .to_string()
}

/// 解析 `hicc::import_class!` 块内容 → `ParsedClassBlock`
///
/// 格式：
/// ```
///     #[cpp(class = "Foo")]
///     class Foo {
///         #[cpp(method = "int get() const")]
///         fn get(&self) -> i32;
///
///     }
/// ```
/// 也支持 `#[interface]` 和 `#[cpp(class = "Foo", destroy = "foo_del")]`。
fn parse_class_content(inner_lines: &[String]) -> Option<ParsedClassBlock> {
    let mut class_name = String::new();
    let mut class_attr = String::new();
    let mut methods: Vec<BlockMethod> = Vec::new();
    let mut pending_attr: Option<String> = None;
    let mut in_class_body = false;
    let mut class_body_depth = 0i32;

    for line in inner_lines {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if !in_class_body {
            // `#[cpp(class = "...")]` 或 `#[cpp(class = "...", destroy = "...")]`
            if trimmed.starts_with("#[cpp(class") {
                class_attr = trimmed.to_string();
                if let Some(name) = extract_quoted_value(trimmed, "class = ") {
                    class_name = name;
                }
                continue;
            }
            // `#[interface]` 属性行：类名从后续的 `class Foo {` 行提取
            if trimmed == "#[interface]" {
                class_attr = trimmed.to_string();
                continue;
            }
            // `class Foo {` 或 `pub class Foo {` 行（codegen 会生成 pub class）
            let class_part = trimmed.strip_prefix("pub ").unwrap_or(trimmed);
            if class_part.starts_with("class ") && trimmed.contains('{') {
                // 若是 interface 类，从这里提取类名
                if class_name.is_empty() {
                    class_name = extract_class_name_from_line(class_part);
                }
                in_class_body = true;
                class_body_depth = 1;
                continue;
            }
            // `class Foo {}` 或 `pub class Foo {}` 空类
            if class_part.starts_with("class ") && trimmed.ends_with('}') {
                if class_name.is_empty() {
                    class_name = extract_class_name_from_line(class_part);
                }
                break;
            }
        } else {
            // 在类体内
            class_body_depth += count_brace_delta(trimmed);

            if class_body_depth <= 0 {
                // 类体结束
                break;
            }

            // 方法属性行
            if trimmed.starts_with("#[cpp(method") {
                pending_attr = Some(trimmed.to_string());
                continue;
            }

            // 方法签名行（以 `fn `、`pub fn `、`unsafe fn `、`pub unsafe fn ` 开头）
            if trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("unsafe fn ")
                || trimmed.starts_with("pub unsafe fn ")
            {
                if let Some(attr) = pending_attr.take() {
                    methods.push(BlockMethod {
                        attr,
                        fn_sig: trimmed.to_string(),
                    });
                }
                continue;
            }
        }
    }

    if class_name.is_empty() {
        None
    } else {
        // 若类名为模板特化（含 '<'），提取基类名（'<' 之前的部分）
        let template_base = if class_name.contains('<') {
            Some(
                class_name
                    .split('<')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            )
        } else {
            None
        };
        Some(ParsedClassBlock {
            class_name,
            class_attr,
            methods,
            template_base,
        })
    }
}

/// 解析 `hicc::import_lib!` 块内容 → `ParsedLibBlock`
///
/// 格式：
/// ```
///     #![link_name = "foo"]
///
///     class Foo;
///
///     #[cpp(func = "Foo* foo_new()")]
///     fn foo_new() -> *mut Foo;
/// ```
fn parse_lib_content(inner_lines: &[String]) -> Option<ParsedLibBlock> {
    let mut link_name = String::new();
    let mut fwd_decls: Vec<String> = Vec::new();
    let mut fn_bindings: Vec<ParsedFnBinding> = Vec::new();
    let mut pending_attr: Option<String> = None;

    for line in inner_lines {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        // `#![link_name = "..."]`
        if trimmed.starts_with("#![link_name") {
            if let Some(name) = extract_quoted_value(trimmed, "link_name = ") {
                link_name = name;
            }
            continue;
        }

        // 前向声明：`class Foo;` 或 `struct Foo;`
        if (trimmed.starts_with("class ") || trimmed.starts_with("struct "))
            && trimmed.ends_with(';')
            && !trimmed.contains('(')
        {
            fwd_decls.push(trimmed.to_string());
            continue;
        }

        // 函数属性行
        if trimmed.starts_with("#[cpp(func") {
            pending_attr = Some(trimmed.to_string());
            continue;
        }

        // 函数签名行（以 `fn `、`pub fn `、`unsafe fn `、`pub unsafe fn ` 开头）
        if trimmed.starts_with("fn ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("unsafe fn ")
            || trimmed.starts_with("pub unsafe fn ")
        {
            if let Some(attr) = pending_attr.take() {
                fn_bindings.push(ParsedFnBinding {
                    attr,
                    fn_sig: trimmed.to_string(),
                });
            }
            continue;
        }
    }

    if link_name.is_empty() {
        None
    } else {
        Some(ParsedLibBlock {
            link_name,
            fwd_decls,
            fn_bindings,
        })
    }
}

// ─────────────────────────────────────────────
//  工具函数
// ─────────────────────────────────────────────

/// 从形如 `#[cpp(class = "Foo")]` 的行中提取 `key` 后的引号值。
fn extract_quoted_value(line: &str, key: &str) -> Option<String> {
    let pos = line.find(key)?;
    let rest = &line[pos + key.len()..];
    let start = rest.find('"')? + 1;
    let end = rest[start..].find('"')?;
    Some(rest[start..start + end].to_string())
}

// ─────────────────────────────────────────────
//  模板类体提取（用于跨翻译单元块级去重）
// ─────────────────────────────────────────────

/// 从 `hicc::cpp!` 内容行中提取所有完整模板类体的规范化字符串。
///
/// 规范化规则：去掉每行行尾空白，过滤空行，用 `\n` 拼接。
fn extract_template_bodies(lines: &[String]) -> Vec<String> {
    detect_template_body_ranges(lines)
        .into_iter()
        .map(|(_, _, key)| key)
        .collect()
}

/// 检测 `cpp_lines` 中完整模板类体的行范围和规范化键。
///
/// 返回 `(start_line_idx, end_line_idx_inclusive, normalized_key)`。
/// 用于块级去重：相同规范化键代表逻辑上等价的模板定义。
///
/// # 检测规则
/// - 以 `template` 开头且包含 `<` 的行为模板起始行
/// - 其后（同行或向后最多 3 行）出现 `class`/`struct` 时，认定为模板类
/// - 收集从模板起始行到匹配的闭括号行（`}` 使括号深度归零）
pub(super) fn detect_template_body_ranges(lines: &[String]) -> Vec<(usize, usize, String)> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let t = lines[i].trim();

        // 识别模板起始行：以 "template" 开头且含 "<"
        if !t.starts_with("template") || !t.contains('<') {
            i += 1;
            continue;
        }

        // 判断是否为模板类/结构体
        let has_class_same_line = t.contains(" class ") || t.contains(" struct ");
        let next_has_class = if !has_class_same_line {
            let mut found = false;
            for nt in lines[(i + 1)..lines.len().min(i + 4)]
                .iter()
                .map(|l| l.trim())
            {
                if !nt.is_empty() {
                    found = nt.starts_with("class ") || nt.starts_with("struct ");
                    break;
                }
            }
            found
        } else {
            false
        };

        if !has_class_same_line && !next_has_class {
            i += 1;
            continue;
        }

        // 收集整个模板类体：等待括号深度从 >0 降回 0
        let start = i;
        let mut depth = 0i32;
        let mut entered = false;
        let mut end_idx = None;
        let mut j = i;

        while j < lines.len() {
            for c in lines[j].trim().chars() {
                match c {
                    '{' => {
                        depth += 1;
                        entered = true;
                    }
                    '}' => {
                        depth -= 1;
                        if entered && depth == 0 {
                            end_idx = Some(j);
                        }
                    }
                    _ => {}
                }
            }
            if end_idx.is_some() {
                break;
            }
            j += 1;
        }

        if let Some(end) = end_idx {
            // 规范化：去掉行尾空白，过滤空行，换行拼接
            let normalized: String = lines[start..=end]
                .iter()
                .map(|l| l.trim_end())
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            result.push((start, end, normalized));
            i = end + 1;
        } else {
            i += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"hicc::cpp! {
    #include "foo.h"

    Foo* foo_new(int v) { return new Foo(v); }
}

hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;

    }
}

hicc::import_lib! {
    #![link_name = "foo"]

    class Foo;

    #[cpp(func = "Foo* foo_new(int v)")]
    fn foo_new(v: i32) -> *mut Foo;
}
"#;

    #[test]
    fn parse_cpp_block() {
        let unit = parse_unit_rs(SAMPLE);
        assert!(unit
            .cpp_lines
            .iter()
            .any(|l| l.contains("#include \"foo.h\"")));
        assert!(unit.cpp_lines.iter().any(|l| l.contains("foo_new")));
    }

    #[test]
    fn parse_class_block() {
        let unit = parse_unit_rs(SAMPLE);
        assert_eq!(unit.class_blocks.len(), 1);
        let cb = &unit.class_blocks[0];
        assert_eq!(cb.class_name, "Foo");
        assert_eq!(cb.methods.len(), 1);
        assert!(cb.methods[0].attr.contains("int get() const"));
        assert!(cb.methods[0].fn_sig.contains("fn get"));
    }

    #[test]
    fn parse_lib_block() {
        let unit = parse_unit_rs(SAMPLE);
        let lb = unit.lib_block.as_ref().unwrap();
        assert_eq!(lb.link_name, "foo");
        assert_eq!(lb.fwd_decls.len(), 1);
        assert_eq!(lb.fn_bindings.len(), 1);
        assert!(lb.fn_bindings[0].attr.contains("foo_new"));
    }

    #[test]
    fn extract_quoted_value_works() {
        assert_eq!(
            extract_quoted_value("#[cpp(class = \"Foo\")]", "class = "),
            Some("Foo".to_string())
        );
        assert_eq!(
            extract_quoted_value("#![link_name = \"hello\"]", "link_name = "),
            Some("hello".to_string())
        );
    }

    // ── count_brace_delta 边界测试 ────────────────────────────────────

    #[test]
    fn brace_delta_plain() {
        assert_eq!(count_brace_delta("{"), 1);
        assert_eq!(count_brace_delta("}"), -1);
        assert_eq!(count_brace_delta("{}"), 0);
        assert_eq!(count_brace_delta("{{}}"), 0);
    }

    #[test]
    fn brace_delta_nested_braces() {
        // 嵌套花括号正确累加
        assert_eq!(count_brace_delta("{ { } }"), 0);
        assert_eq!(count_brace_delta("{ {"), 2);
        assert_eq!(count_brace_delta("} }"), -2);
    }

    #[test]
    fn brace_delta_string_literal_ignored() {
        // 字符串字面量内的 { 和 } 不应影响计数
        assert_eq!(count_brace_delta(r#"let s = "{ not a brace }";"#), 0);
        assert_eq!(count_brace_delta(r#"let s = "{{}}";"#), 0);
    }

    #[test]
    fn brace_delta_escaped_quote_in_string() {
        // 转义引号不影响字符串状态
        assert_eq!(count_brace_delta(r#"let s = "\"{";"#), 0);
    }

    #[test]
    fn brace_delta_macro_call_ignored() {
        // `macro!{` 形式的 `{` 由外层计数，此处不计入
        assert_eq!(count_brace_delta("hicc::import_class!{"), 0);
        // 闭合的 `}` 仍然计入
        assert_eq!(count_brace_delta("}"), -1);
    }

    // ── 多个连续宏块提取测试 ─────────────────────────────────────────

    const MULTI_CLASS: &str = r#"hicc::import_class! {
    #[cpp(class = "Alpha")]
    class Alpha {
        #[cpp(method = "int val() const")]
        fn val(&self) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Beta")]
    class Beta {
        #[cpp(method = "void run()")]
        fn run(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "multi"]

    class Alpha;
    class Beta;
}
"#;

    #[test]
    fn parse_multiple_class_blocks() {
        let unit = parse_unit_rs(MULTI_CLASS);
        assert_eq!(unit.class_blocks.len(), 2, "应解析出 2 个 import_class! 块");
        let names: Vec<&str> = unit
            .class_blocks
            .iter()
            .map(|b| b.class_name.as_str())
            .collect();
        assert!(names.contains(&"Alpha"), "应包含 Alpha 类");
        assert!(names.contains(&"Beta"), "应包含 Beta 类");
    }

    #[test]
    fn parse_lib_block_with_multiple_fwd_decls() {
        let unit = parse_unit_rs(MULTI_CLASS);
        let lb = unit.lib_block.as_ref().expect("应有 import_lib! 块");
        assert_eq!(lb.link_name, "multi");
        assert_eq!(lb.fwd_decls.len(), 2, "应有 Alpha 和 Beta 两个前向声明");
    }
}
