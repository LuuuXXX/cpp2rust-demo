# Clang AST 解析能力示例

## 1. 概述

本文档展示 Clang AST（Abstract Syntax Tree）能够解析的 C++ 类型信息，以及我们如何使用这些信息来生成 Rust FFI 代码。

## 2. Clang AST JSON 节点类型

Clang 可以将 C++ 代码解析为 JSON 格式的 AST。以下是我们 v1 版本重点关注的 AST 节点类型：

| AST 节点类型 | JSON kind 值 | 解析目标 |
|-------------|--------------|----------|
| `CXXRecordDecl` | `"CXXRecordDecl"` | 类/结构体定义 |
| `CXXMethodDecl` | `"CXXMethodDecl"` | 类方法 |
| `FunctionDecl` | `"FunctionDecl"` | 函数声明 |
| `EnumDecl` | `"EnumDecl"` | 枚举定义 |
| `EnumConstantDecl` | `"EnumConstantDecl"` | 枚举值 |
| `NamespaceDecl` | `"NamespaceDecl"` | 命名空间 |
| `FieldDecl` | `"FieldDecl"` | 类成员字段 |
| `ParmVarDecl` | `"ParmVarDecl"` | 函数参数 |
| `VarDecl` | `"VarDecl"` | 变量/静态成员 |
| `TypedefDecl` | `"TypedefDecl"` | 类型别名 |

## 3. 具体示例

### 3.1 嵌套命名空间类

**C++ 源码**：
```cpp
namespace foo {
    namespace bar {
        namespace config {
            class ConfigManager {
            public:
                static constexpr size_t MAX_ENTRIES = 10;
            private:
                int values_[MAX_ENTRIES];
                const char* keys_[MAX_ENTRIES];
                size_t count_;
            public:
                ConfigManager();
                ~ConfigManager();
                void set_value(const char* key, int value);
                int get_value(const char* key) const;
            };
        }
    }
}
```

**对应的 Clang AST JSON 结构**：

```json
{
  "id": "0x123",
  "kind": "NamespaceDecl",
  "name": "foo",
  "inner": [
    {
      "id": "0x124",
      "kind": "NamespaceDecl",
      "name": "bar",
      "inner": [
        {
          "id": "0x125",
          "kind": "NamespaceDecl",
          "name": "config",
          "inner": [
            {
              "id": "0x126",
              "kind": "CXXRecordDecl",
              "name": "ConfigManager",
              "tagUsed": "class",
              "completeDefinition": true,
              "inner": [
                {
                  "id": "0x130",
                  "kind": "AccessSpecDecl",
                  "access": "public"
                },
                {
                  "id": "0x131",
                  "kind": "VarDecl",
                  "name": "MAX_ENTRIES",
                  "type": {"qualType": "const int"},
                  "storageClass": "static",
                  "constexpr": true
                },
                {
                  "id": "0x132",
                  "kind": "AccessSpecDecl",
                  "access": "private"
                },
                {
                  "id": "0x133",
                  "kind": "FieldDecl",
                  "name": "values_",
                  "type": {"qualType": "int[10]"}
                },
                {
                  "id": "0x134",
                  "kind": "FieldDecl",
                  "name": "keys_",
                  "type": {"qualType": "const char *[10]"}
                },
                {
                  "id": "0x135",
                  "kind": "FieldDecl",
                  "name": "count_",
                  "type": {"qualType": "size_t"}
                },
                {
                  "id": "0x140",
                  "kind": "CXXConstructorDecl",
                  "name": "ConfigManager",
                  "isImplicit": false
                },
                {
                  "id": "0x141",
                  "kind": "CXXDestructorDecl",
                  "name": "~ConfigManager",
                  "isImplicit": false
                },
                {
                  "id": "0x142",
                  "kind": "CXXMethodDecl",
                  "name": "set_value",
                  "type": {"qualType": "void (const char *, int)"},
                  "inner": [
                    {
                      "kind": "ParmVarDecl",
                      "name": "key",
                      "type": {"qualType": "const char *"}
                    },
                    {
                      "kind": "ParmVarDecl",
                      "name": "value",
                      "type": {"qualType": "int"}
                    }
                  ]
                },
                {
                  "id": "0x143",
                  "kind": "CXXMethodDecl",
                  "name": "get_value",
                  "type": {"qualType": "int (const char *) const"},
                  "const": true,
                  "inner": [
                    {
                      "kind": "ParmVarDecl",
                      "name": "key",
                      "type": {"qualType": "const char *"}
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

**提取的信息**：
```rust
struct ClassInfo {
    name: "ConfigManager",
    full_name: "foo::bar::config::ConfigManager",
    namespace_depth: 3,  // foo -> bar -> config = 3层
    methods: vec![
        MethodInfo { name: "ConfigManager", kind: Constructor },
        MethodInfo { name: "~ConfigManager", kind: Destructor },
        MethodInfo { name: "set_value", params: [("key", "const char *"), ("value", "int")] },
        MethodInfo { name: "get_value", params: [("key", "const char *)"], const: true },
    ],
    static_fields: vec![
        FieldInfo { name: "MAX_ENTRIES", type: "const int" },
    ],
    fields: vec![
        FieldInfo { name: "values_", type: "int[10]" },
        FieldInfo { name: "keys_", type: "const char *[10]" },
        FieldInfo { name: "count_", type: "size_t" },
    ],
}
```

### 3.2 枚举类

**C++ 源码**：
```cpp
namespace example {
enum class ErrorCode : int {
    None = 0,
    InvalidInput = 1,
    OutOfMemory = 2,
    NotFound = 3,
    Unknown = 99
};

enum class State : unsigned char {
    Idle = 0,
    Running = 1,
    Paused = 2,
    Stopped = 3
};
}
```

**对应的 Clang AST JSON 结构**：
```json
{
  "kind": "EnumDecl",
  "name": "ErrorCode",
  "inner": [
    {"kind": "EnumConstantDecl", "name": "None", "value": {"value": 0}},
    {"kind": "EnumConstantDecl", "name": "InvalidInput", "value": {"value": 1}},
    {"kind": "EnumConstantDecl", "name": "OutOfMemory", "value": {"value": 2}},
    {"kind": "EnumConstantDecl", "name": "NotFound", "value": {"value": 3}},
    {"kind": "EnumConstantDecl", "name": "Unknown", "value": {"value": 99}}
  ]
}
```

**提取的信息**：
```rust
struct EnumInfo {
    name: "ErrorCode",
    full_name: "example::ErrorCode",
    underlying_type: "int",
    is_enum_class: true,
    values: vec![
        ("None", 0),
        ("InvalidInput", 1),
        ("OutOfMemory", 2),
        ("NotFound", 3),
        ("Unknown", 99),
    ],
}
```

### 3.3 普通函数

**C++ 源码**：
```cpp
int global_func(const char* str);
void process_data(int* data, size_t len);
const char* get_version();
```

**对应的 Clang AST JSON**：
```json
[
  {
    "kind": "FunctionDecl",
    "name": "global_func",
    "type": {"qualType": "int (const char *)"},
    "inner": [
      {"kind": "ParmVarDecl", "name": "str", "type": {"qualType": "const char *"}}
    ]
  },
  {
    "kind": "FunctionDecl",
    "name": "process_data",
    "type": {"qualType": "void (int *, size_t)"},
    "inner": [
      {"kind": "ParmVarDecl", "name": "data", "type": {"qualType": "int *"}},
      {"kind": "ParmVarDecl", "name": "len", "type": {"qualType": "size_t"}}
    ]
  },
  {
    "kind": "FunctionDecl",
    "name": "get_version",
    "type": {"qualType": "const char *(void)"}
  }
]
```

**提取的信息**：
```rust
struct FunctionInfo {
    name: "global_func",
    return_type: "int",
    params: vec![("str", "const char *")],
}

struct FunctionInfo {
    name: "process_data",
    return_type: "void",
    params: vec![("data", "int *"), ("len", "size_t")],
}

struct FunctionInfo {
    name: "get_version",
    return_type: "const char *",
    params: vec![],
}
```

## 4. Rust FFI 生成映射

### 4.1 类型映射表

| C++ 类型 | Clang qualType | Rust 类型 |
|----------|----------------|----------|
| `int` | `"int"` | `i32` |
| `unsigned int` | `"unsigned int"` | `u32` |
| `char` | `"char"` | `i8` |
| `const char*` | `"const char *"` | `*const i8` |
| `int*` | `"int *"` | `*mut i32` |
| `size_t` | `"size_t"` | `usize` |
| `void` | `"void"` | `()` |
| `int[10]` | `"int[10]"` | `[i32; 10]` |
| `int ()(int)` | `"int (int)"` | `fn(i32) -> i32` |

### 4.2 函数签名生成

对于 `void set_value(const char* key, int value)`：

**hicc import_lib 格式**：
```rust
#[link(name = "libname")]
unsafe extern "C" {
    fn config_manager_set_value(p: ConfigManager, key: *const i8, value: i32);
}
```

**hicc import_class 格式**（仅适用于非嵌套命名空间类）：
```rust
hicc::import_class! {
    #[cpp(class = "ConfigManager")]
    class ConfigManager {
        #[cpp(method = "void set_value(const char* key, int value)")]
        fn set_value(&mut self, key: *const i8, value: i32);
    }
}
```

## 5. 嵌套命名空间判断

```rust
fn calculate_namespace_depth(full_name: &str) -> usize {
    // "foo::bar::config::ConfigManager" -> depth = 3
    full_name.matches("::").count().saturating_sub(1)
}

fn needs_opaque_pointer(full_name: &str) -> bool {
    // 嵌套命名空间深度 >= 2 时，使用 void* 模式
    calculate_namespace_depth(full_name) >= 2
}

// 示例
assert_eq!(calculate_namespace_depth("foo::bar::config::ConfigManager"), 3);
assert!(needs_opaque_pointer("foo::bar::config::ConfigManager"));

assert_eq!(calculate_namespace_depth("foo::bar::DataProcessor"), 2);
assert!(needs_opaque_pointer("foo::bar::DataProcessor"));

assert_eq!(calculate_namespace_depth("foo::ConfigManager"), 1);
assert!(!needs_opaque_pointer("foo::ConfigManager"));

assert_eq!(calculate_namespace_depth("ConfigManager"), 0);
assert!(!needs_opaque_pointer("ConfigManager"));
```

## 6. 实际代码解析参考

c2rust-demo 项目中的 AST 解析实现参考：

- **`clang_ast` crate**：自定义的 Clang AST JSON 节点定义
- **`src/split/file.rs`**：完整的 AST 节点类型定义（Kind enum）

关键类型定义：
```rust
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Kind {
    EnumDecl(EnumDecl),
    RecordDecl(RecordDecl),
    FunctionDecl(FunctionDecl),
    VarDecl(VarDecl),
    TypedefDecl(TypedefDecl),
    TranslationUnitDecl(TranslationUnitDecl),
    CompoundStmt,
    Other(OtherDecl),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    loc: SourceLocation,
    range: SourceRange,
    #[serde(rename = "type")]
    ty: MyClangType,
    #[serde(default)]
    storage_class: Option<String>,
    #[serde(default, rename = "isImplicit")]
    is_implicit: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MyClangType {
    #[serde(rename = "qualType")]
    qual_type: String,
    #[serde(rename = "desugaredQualType")]
    desugared_qual_type: Option<String>,
}
```

## 7. 总结

Clang AST 能够完整解析以下 C++ 特性：

| 特性 | 支持状态 | 说明 |
|------|----------|------|
| 类定义 | ✅ | CXXRecordDecl |
| 类方法 | ✅ | CXXMethodDecl |
| 构造/析构函数 | ✅ | CXXConstructorDecl/CXXDestructorDecl |
| 命名空间 | ✅ | NamespaceDecl |
| 枚举类 | ✅ | EnumDecl + EnumConstantDecl |
| 函数声明 | ✅ | FunctionDecl |
| 静态成员 | ✅ | VarDecl with storageClass="static" |
| 实例成员 | ✅ | FieldDecl |
| 函数参数 | ✅ | ParmVarDecl |

这些信息足够生成 v1 版本的 Rust FFI 代码。
