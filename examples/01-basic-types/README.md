# 示例 01：基础类型与函数

## 特性概述

本示例展示 C++ 的**基础类型系统**，包括整数类型、浮点类型、布尔类型、枚举、结构体以及全局函数。这些是 C++ 中最基本的构建块，也是 hicc FFI 映射的起点。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| 基础算术类型 | `int`、`double`、`bool` 等原生类型 |
| 枚举 `enum` | 整数枚举类型（`Color`） |
| 结构体 `struct` | 聚合数据类型（`Point3D`） |
| 全局函数 | 无类作用域的自由函数 |
| 常量 `const` | 编译期常量 |

### 代码结构

```cpp
// 枚举类型
enum Color { RED = 0, GREEN = 1, BLUE = 2 };

// 结构体
struct Point3D { double x, y, z; };

// 全局函数
int add_int(int a, int b);
double add_double(double a, double b);
int sum_array(const int* arr, int len);
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

AST 中的关键节点类型：

| AST 节点 | 对应 C++ 构造 |
|----------|--------------|
| `EnumDecl` | `enum Color` 枚举声明 |
| `EnumConstantDecl` | `RED`、`GREEN`、`BLUE` 枚举值 |
| `RecordDecl` | `struct Point3D` 结构体声明 |
| `FieldDecl` | `x`、`y`、`z` 结构体字段 |
| `FunctionDecl` | 全局函数声明 |
| `ParmVarDecl` | 函数参数 |
| `VarDecl` | `const int MAX_VALUE` 等常量 |

AST 片段示例：

```json
{
  "kind": "FunctionDecl",
  "name": "add_int",
  "type": { "qualType": "int (int, int)" },
  "inner": [
    { "kind": "ParmVarDecl", "name": "a", "type": { "qualType": "int" } },
    { "kind": "ParmVarDecl", "name": "b", "type": { "qualType": "int" } }
  ]
}
```

## hicc 处理方式

### 基础类型映射

cpp2rust-demo 根据 AST 中的 `qualType` 字段将 C++ 类型映射为 Rust 类型：

| C++ 类型 | Rust 类型 |
|----------|-----------|
| `int` | `i32` |
| `long` | `i64` |
| `unsigned int` | `u32` |
| `double` | `f64` |
| `float` | `f32` |
| `bool` | `bool` |
| `const char*` | `*const i8` |
| `void` | `()` |

### 全局函数 → `import_lib!`

所有全局函数（`FunctionDecl` 节点）通过 `hicc::import_lib!` 宏映射：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "int add_int(int, int)")]
    fn add_int(a: i32, b: i32) -> i32;

    #[cpp(func = "double add_double(double, double)")]
    fn add_double(a: f64, b: f64) -> f64;

    #[cpp(func = "int sum_array(const int*, int)")]
    fn sum_array(arr: *const i32, len: i32) -> i32;
}
```

### 枚举映射

C++ `enum` 类型会生成对应的 Rust 枚举类型定义，值映射为整数常量：

```rust
#[repr(i32)]
pub enum Color {
    Red = 0,
    Green = 1,
    Blue = 2,
}
```

### 结构体映射

`struct` 聚合类型通过 `import_class!` 处理，生成对应的 Rust struct：

```rust
hicc::import_class! {
    #[cpp(class = "Point3D")]
    class Point3D {}
}
```

### build.rs 配置

```rust
fn main() {
    hicc_build::Build::new().rust_file("src/main.rs").compile("example");
    println!("cargo::rustc-link-lib=example");
    println!("cargo::rustc-link-lib=stdc++");
    println!("cargo::rerun-if-changed=src/main.rs");
}
```

## 注意事项

1. **数组参数**：`const int* arr` 在 Rust 侧映射为 `*const i32`，调用方负责管理生命周期
2. **枚举整数转换**：`color_to_int` 等辅助函数可以直接通过 `import_lib!` 映射
3. **浮点精度**：C++ `double` → Rust `f64`，`float` → `f32`，需确保类型匹配
