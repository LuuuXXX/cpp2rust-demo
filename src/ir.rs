use serde::{Deserialize, Serialize};

/// 单个头文件的解析结果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParsedHeader {
    pub header_name: String,
    pub include_path: String,
    pub functions: Vec<Function>,
    pub classes: Vec<Class>,
    /// 从 `typedef` 提取出的类型别名（如函数指针 typedef）。
    pub typedefs: Vec<TypedefAlias>,
}

/// 顶层可导出的 C/C++ 兼容函数。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub rust_name: String,
    pub return_type: String,
    pub params: Vec<Parameter>,
    pub kind: FunctionKind,
    pub explicit_void: bool,
    /// 是否为 C++ 友元函数（在某个类体中以 `friend` 声明）。
    #[serde(default)]
    pub is_friend: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FunctionKind {
    Free,
    Constructor {
        class_name: String,
    },
    Destructor {
        class_name: String,
    },
    StaticMethodShim {
        class_name: String,
        method_name: String,
    },
}

/// C++ 类。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Method>,
}

/// 类成员函数。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Method {
    pub name: String,
    pub rust_name: String,
    pub return_type: Option<String>,
    pub params: Vec<Parameter>,
    pub kind: MethodKind,
    pub is_const: bool,
    pub is_static: bool,
    /// 是否为运算符重载（如 operator+, operator++）。
    pub is_operator: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MethodKind {
    Constructor,
    Destructor,
    Regular,
}

/// 参数定义。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub cpp_type: String,
}

/// typedef 别名，目前主要用于函数指针类型。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypedefAlias {
    /// C++ typedef 名称（如 `IntBinaryOp`）。
    pub name: String,
    /// 对应的 Rust 类型表示（如 `extern "C" fn(i32, i32) -> i32`）。
    pub rust_type: String,
}
