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
    /// 所有 `hicc::import_class!` 块
    pub class_blocks: Vec<ParsedClassBlock>,
    /// `hicc::import_lib!` 块（最多一个；多个时取最后一个合并结果不变）
    pub lib_block: Option<ParsedLibBlock>,
}

/// 单个 `hicc::import_class!` 块
#[derive(Debug)]
pub struct ParsedClassBlock {
    pub class_name: String,
    pub methods: Vec<BlockMethod>,
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

/// 解析一个 unit `.rs` 文件的内容，提取三类 hicc 块。
pub fn parse_unit_rs(src: &str) -> ParsedUnit {
    let mut unit = ParsedUnit::default();

    // 先将源码切分为顶层 hicc 块
    let raw_blocks = extract_raw_blocks(src);

    for rb in raw_blocks {
        match rb.kind {
            BlockKind::Cpp => {
                unit.cpp_lines = parse_cpp_content(&rb.inner_lines);
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
                    blocks.push(RawBlock { kind, inner_lines: inner_lines.clone() });
                }
                inner_lines.clear();
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
            if l.starts_with("    ") { l[4..].to_string() } else { l.trim_end().to_string() }
        })
        .collect();

    // 去掉首尾空行
    let start = lines.iter().position(|l| !l.is_empty()).unwrap_or(lines.len());
    let end = lines.iter().rposition(|l| !l.is_empty()).map(|i| i + 1).unwrap_or(0);
    if start >= end { Vec::new() } else { lines[start..end].to_vec() }
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
fn parse_class_content(inner_lines: &[String]) -> Option<ParsedClassBlock> {
    let mut class_name = String::new();
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
            // 寻找 `#[cpp(class = "...")]` 属性行
            if trimmed.starts_with("#[cpp(class") {
                // 提取类名
                if let Some(name) = extract_quoted_value(trimmed, "class = ") {
                    class_name = name;
                }
                continue;
            }
            // 寻找 `class Foo {` 行
            if trimmed.starts_with("class ") && trimmed.contains('{') {
                in_class_body = true;
                class_body_depth = 1;
                continue;
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

            // 方法签名行（以 `fn ` 开头）
            if trimmed.starts_with("fn ") {
                if let Some(attr) = pending_attr.take() {
                    methods.push(BlockMethod { attr, fn_sig: trimmed.to_string() });
                }
                continue;
            }
        }
    }

    if class_name.is_empty() {
        None
    } else {
        Some(ParsedClassBlock { class_name, methods })
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

        // 函数签名行（以 `fn ` 或 `unsafe fn ` 开头）
        if trimmed.starts_with("fn ") || trimmed.starts_with("unsafe fn ") {
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
        Some(ParsedLibBlock { link_name, fwd_decls, fn_bindings })
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
        assert!(unit.cpp_lines.iter().any(|l| l.contains("#include \"foo.h\"")));
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
}
