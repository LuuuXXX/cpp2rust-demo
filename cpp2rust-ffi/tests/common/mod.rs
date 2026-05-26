use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

/// 项目根目录（相对于测试文件）
pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

/// 示例目录
pub fn examples_dir() -> PathBuf {
    repo_root().join("examples")
}

/// 某个示例的目录
pub fn example_dir(name: &str) -> PathBuf {
    examples_dir().join(name)
}

/// 读取某个示例的黄金文件（rust_hicc/src/main.rs）
pub fn read_golden(example: &str) -> String {
    let path = example_dir(example).join("rust_hicc").join("src").join("main.rs");
    fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Golden file not found: {}", path.display()))
}

/// 规范化运行输出（用于 L3 测试比较）：
/// - 去掉行尾空白
/// - 规范化行尾
/// - 替换内存地址为占位符（非确定性输出）
pub fn normalize_output(content: &str) -> String {
    replace_addresses(content)
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

/// 规范化代码内容（用于黄金文件比较）：
/// - 去掉空白行
/// - 去掉行尾空白
/// - 规范化行尾
/// - 跳过纯注释行（以 // 开头的行）- 黄金文件可能有工具未生成的说明注释
pub fn normalize(content: &str) -> String {
    content
        .lines()
        .map(|line| line.trim_end())
        .filter(|line| !line.is_empty())
        .filter(|line| !line.trim().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 替换字符串中的内存地址
fn replace_addresses(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // 检测 0x 前缀
        if i + 1 < bytes.len() && bytes[i] == b'0' && bytes[i + 1] == b'x' {
            let start = i;
            i += 2;
            let addr_start = i;
            while i < bytes.len() && bytes[i].is_ascii_hexdigit() {
                i += 1;
            }
            if i - addr_start >= 4 {
                // 足够长，认为是地址
                result.push_str("0xADDR");
            } else {
                // 太短，保留原始
                result.push_str(&s[start..i]);
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
}

/// 在给定目录运行 cargo build
pub fn cargo_build(dir: &Path) -> std::process::ExitStatus {
    Command::new("cargo")
        .args(["build"])
        .current_dir(dir)
        .status()
        .expect("Failed to run cargo build")
}

/// 在给定目录运行 cargo run，返回 stdout
pub fn cargo_run(dir: &Path) -> String {
    let output = Command::new("cargo")
        .args(["run"])
        .current_dir(dir)
        .output()
        .expect("Failed to run cargo run");
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// 从 README.md 中提取"运行结果"代码块
pub fn parse_readme_run_result(example: &str) -> String {
    let readme_path = example_dir(example).join("README.md");
    let content = fs::read_to_string(&readme_path)
        .unwrap_or_else(|_| panic!("README not found: {}", readme_path.display()));

    // 找到 ## 运行结果 章节
    let start_marker = "## 运行结果";
    let start = content.find(start_marker).unwrap_or_else(|| {
        panic!("'运行结果' section not found in {}", readme_path.display())
    });

    let section = &content[start + start_marker.len()..];

    // 找到第一个代码块
    let code_start = section.find("```").unwrap_or_else(|| {
        panic!("No code block in '运行结果' section of {}", readme_path.display())
    });
    let code_content = &section[code_start + 3..];

    // 跳过语言标识符行（如果有）
    let code_body = if let Some(newline) = code_content.find('\n') {
        let first_line = code_content[..newline].trim();
        if first_line.is_empty() || first_line.chars().all(|c| c.is_alphanumeric()) {
            &code_content[newline + 1..]
        } else {
            code_content
        }
    } else {
        code_content
    };

    let code_end = code_body.find("```").unwrap_or(code_body.len());
    code_body[..code_end].trim().to_string()
}
