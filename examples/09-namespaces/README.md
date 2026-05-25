# 示例 09：命名空间

## 特性概述

本示例展示 C++ 的**命名空间**机制，包括基础命名空间、嵌套命名空间、内联命名空间（用于版本管理）以及匿名命名空间。命名空间是 C++ 代码组织的重要工具，hicc 通过完整限定名（fully-qualified name）保持命名空间信息。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| 命名空间 `namespace` | 基础作用域隔离 |
| 嵌套命名空间 | `namespace Outer::Inner`（C++17） |
| 内联命名空间 `inline namespace` | 允许无前缀访问，常用于版本管理 |
| 匿名命名空间 | 文件作用域内部链接 |
| 限定名访问 | `Math::add()`、`Outer::Inner::nested_function()` |

### 代码结构

```cpp
// 基础命名空间
namespace Math {
    const double PI = 3.14159265358979323846;
    int add(int a, int b);
    double circle_area(double radius);

    namespace Advanced {
        double sin(double x);
        double cos(double x);
    }
}

// 嵌套命名空间
namespace Outer {
    namespace Inner {
        int nested_value = 42;
        int nested_function(int x);
    }
}

// 内联命名空间（版本管理）
namespace Version {
    inline namespace v1 {
        int version_function();  // 可通过 Version::version_function() 访问
    }
    namespace v2 {
        int version_function();  // 必须通过 Version::v2::version_function() 访问
    }
}
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 节点：

| AST 节点 / 属性 | 含义 |
|-----------------|------|
| `NamespaceDecl` | 命名空间声明 |
| `NamespaceDecl.isInline: true` | 内联命名空间 |
| `NamespaceDecl.isAnonymous: true` | 匿名命名空间 |
| `FunctionDecl.name` | 不含命名空间前缀（前缀来自父节点） |

AST 片段示例：

```json
{
  "kind": "NamespaceDecl",
  "name": "Math",
  "inner": [
    { "kind": "VarDecl", "name": "PI", "type": { "qualType": "const double" } },
    { "kind": "FunctionDecl", "name": "add", "type": { "qualType": "int (int, int)" } },
    {
      "kind": "NamespaceDecl",
      "name": "Advanced",
      "inner": [
        { "kind": "FunctionDecl", "name": "sin" }
      ]
    }
  ]
}
```

## hicc 处理方式

### 命名空间中的函数

cpp2rust-demo 解析 AST 时遍历 `NamespaceDecl` 节点，为其中的函数生成带完整限定名的绑定：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    // 命名空间函数使用完整限定名
    #[cpp(func = "int Math::add(int, int)")]
    fn math_add(a: i32, b: i32) -> i32;

    #[cpp(func = "double Math::circle_area(double)")]
    fn math_circle_area(r: f64) -> f64;

    // 嵌套命名空间
    #[cpp(func = "double Math::Advanced::sin(double)")]
    fn math_sin(x: f64) -> f64;

    // 深度嵌套
    #[cpp(func = "int Outer::Inner::nested_function(int)")]
    fn outer_inner_nested(x: i32) -> i32;
}
```

### 内联命名空间

内联命名空间中的符号可以通过父命名空间直接访问，两种写法都有效：

```rust
// 通过内联命名空间的完整路径
#[cpp(func = "int Version::v1::version_function()")]
fn version_function_v1() -> i32;

// 通过父命名空间（内联命名空间透明）
#[cpp(func = "int Version::version_function()")]
fn version_function() -> i32;
```

### 命名空间中的类

命名空间中的类使用完整限定名：

```rust
hicc::import_class! {
    #[cpp(class = "Math::Calculator")]
    class MathCalculator {
        #[cpp(method = "int Math::Calculator::add(int, int) const")]
        fn add(&self, a: i32, b: i32) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    class MathCalculator;

    #[cpp(func = "std::unique_ptr<Math::Calculator> std::make_unique<Math::Calculator>()")]
    fn calculator_new() -> MathCalculator;
}
```

### Rust 侧命名空间组织

Rust 没有 C++ 的命名空间概念，通常通过模块（`mod`）来组织：

```rust
pub mod math {
    use super::*;

    pub fn add(a: i32, b: i32) -> i32 { math_add(a, b) }
    pub fn circle_area(r: f64) -> f64 { math_circle_area(r) }

    pub mod advanced {
        use super::super::*;
        pub fn sin(x: f64) -> f64 { math_sin(x) }
    }
}
```

## 注意事项

1. **完整限定名**：在 `#[cpp(func = ...)]` 和 `#[cpp(method = ...)]` 中必须使用完整的 C++ 限定名，包括所有命名空间前缀
2. **匿名命名空间**：匿名命名空间中的符号具有内部链接，跨翻译单元不可见，通常无法从 Rust 侧链接
3. **内联命名空间版本冲突**：同名内联命名空间展开后可能产生符号冲突，需在 Rust 侧明确区分版本
4. **`using namespace`**：AST 中不体现 `using namespace` 声明，函数总是通过其定义时的命名空间记录
