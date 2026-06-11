//! Phase A 模板 AST 提取测试。
//!
//! 验证 `ast_parser::parse_preprocessed` 能从模板类 / 模板函数声明中提取
//! 结构化信息（泛型形参 + 成员 / 签名）。这些信息供后续阶段生成泛型
//! `import_class!` / `import_lib!` 绑定使用。
//!
//! 需 libclang 与 C++ 预处理器（clang++ / g++），故以 `full-test` 门禁。

use cpp2rust_demo::ast_parser;
use std::sync::Mutex;

/// libclang 在每个进程内只允许存在一个 `Clang` 实例，并发解析会报
/// "an instance of `Clang` already exists"。用全局锁串行化解析调用，
/// 使本测试无需依赖 `--test-threads=1` 即可稳定运行。
static CLANG_LOCK: Mutex<()> = Mutex::new(());

/// 串行解析预处理文件（持锁期间完成解析，避免与其他测试线程争用 libclang）。
fn parse_locked(pre: &std::path::Path) -> cpp2rust_demo::ast_parser::CppAst {
    let _guard = CLANG_LOCK.lock().unwrap();
    ast_parser::parse_preprocessed(pre).expect("parse failed")
}

/// 将 C++ 源码写入临时 `.cpp`，用 clang++/g++ 预处理为 `.cpp2rust`，返回其路径。
fn preprocess(cpp_src: &str, stem: &str) -> Option<std::path::PathBuf> {
    // 目录名加入进程 ID，避免并发/重复运行时的临时文件冲突。
    let dir = std::env::temp_dir().join(format!("cpp2rust_tmpl_{}_{}", std::process::id(), stem));
    std::fs::create_dir_all(&dir).ok()?;
    let cpp = dir.join(format!("{}.cpp", stem));
    std::fs::write(&cpp, cpp_src).ok()?;
    let out = dir.join(format!("{}.cpp2rust", stem));
    for cxx in ["clang++", "g++"] {
        let ok = std::process::Command::new(cxx)
            .args([
                "-E",
                "-C",
                cpp.to_str().unwrap(),
                "-o",
                out.to_str().unwrap(),
            ])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            return Some(out);
        }
    }
    None
}

#[test]
#[cfg_attr(
    not(feature = "full-test"),
    ignore = "requires libclang + C++ preprocessor; run with --features full-test"
)]
fn extracts_template_class_with_type_param_and_members() {
    let src = r#"
template<typename T>
class Stack {
public:
    int size() const { return 0; }
    bool empty() const { return true; }
    void push(T value) { (void)value; }
    T top() const { return T(); }
    void pop() {}
};
"#;
    let pre = preprocess(src, "stack").expect("preprocess failed");
    let ast = parse_locked(&pre);

    let tc = ast
        .template_classes
        .iter()
        .find(|t| t.name == "Stack")
        .expect("应提取到模板类 Stack");

    // 泛型形参：单个类型形参 T
    assert_eq!(tc.type_params.len(), 1, "Stack 应有 1 个模板形参");
    assert_eq!(tc.type_params[0].name, "T");
    assert_eq!(tc.type_params[0].kind, "type");
    assert!(tc.is_from_current_file, "Stack 应来自当前编译单元");

    // 成员方法：size / empty / push / top / pop
    let method_names: Vec<&str> = tc.methods.iter().map(|m| m.name.as_str()).collect();
    for expected in ["size", "empty", "push", "top", "pop"] {
        assert!(
            method_names.contains(&expected),
            "Stack 应包含方法 {}，实际 {:?}",
            expected,
            method_names
        );
    }

    // 模板类不应进入普通 classes 列表（避免被当作具体类生成）。
    assert!(
        !ast.classes.iter().any(|c| c.name == "Stack"),
        "模板类不应出现在 classes 列表中"
    );
}

#[test]
#[cfg_attr(
    not(feature = "full-test"),
    ignore = "requires libclang + C++ preprocessor; run with --features full-test"
)]
fn extracts_template_function_signature() {
    let src = r#"
template<typename T>
void do_swap(T* a, T* b) {
    T temp = *a;
    *a = *b;
    *b = temp;
}
"#;
    let pre = preprocess(src, "do_swap").expect("preprocess failed");
    let ast = parse_locked(&pre);

    let tf = ast
        .template_functions
        .iter()
        .find(|f| f.name == "do_swap")
        .expect("应提取到模板函数 do_swap");

    assert_eq!(tf.type_params.len(), 1);
    assert_eq!(tf.type_params[0].name, "T");
    assert_eq!(tf.type_params[0].kind, "type");
    assert_eq!(tf.params.len(), 2, "do_swap 应有 2 个参数");
    assert_eq!(tf.return_type, "void");
    assert!(tf.is_from_current_file);
}

#[test]
#[cfg_attr(
    not(feature = "full-test"),
    ignore = "requires libclang + C++ preprocessor; run with --features full-test"
)]
fn extracts_non_type_template_parameter() {
    let src = r#"
template<typename T, int N>
class FixedArray {
public:
    T data[N];
    int capacity() const { return N; }
};
"#;
    let pre = preprocess(src, "fixed_array").expect("preprocess failed");
    let ast = parse_locked(&pre);

    let tc = ast
        .template_classes
        .iter()
        .find(|t| t.name == "FixedArray")
        .expect("应提取到模板类 FixedArray");

    assert_eq!(tc.type_params.len(), 2, "FixedArray 应有 2 个模板形参");
    assert_eq!(tc.type_params[0].name, "T");
    assert_eq!(tc.type_params[0].kind, "type");
    assert_eq!(tc.type_params[1].name, "N");
    assert_eq!(tc.type_params[1].kind, "non_type", "N 应为非类型模板形参");
}
