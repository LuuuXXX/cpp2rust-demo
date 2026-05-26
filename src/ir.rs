use serde::{Deserialize, Serialize};

/// 单个头文件的解析结果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParsedHeader {
    pub header_name: String,
    pub include_path: String,
    pub functions: Vec<Function>,
    pub classes: Vec<Class>,
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
