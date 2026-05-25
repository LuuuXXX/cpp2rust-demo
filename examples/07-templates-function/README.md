# 示例 07：函数模板

## 特性概述

本示例展示 C++ 的**函数模板**，包括基础函数模板、多类型参数模板、非类型模板参数以及函数模板特化。函数模板是 C++ 泛型编程的基础，hicc 对已实例化的模板函数提供部分支持。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| 函数模板 | `template<typename T> T max(T a, T b)` |
| 多类型参数 | `template<typename T, typename U>` |
| 非类型参数 | `template<typename T, size_t N>` |
| 模板特化 | `template<> bool max<bool>(bool, bool)` |
| 可变参数模板 | `template<typename... Args>` |

### 代码结构

```cpp
// 基础函数模板
template<typename T>
T max_value(T a, T b);

template<typename T>
void swap_values(T& a, T& b);

// 非类型参数模板
template<typename T, size_t N>
T sum_array(T (&arr)[N]);

// 多类型参数模板
template<typename T, typename U>
U convert(T value);

// 模板结构体
template<typename T, typename U>
struct Pair { T first; U second; };
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 节点：

| AST 节点 | 含义 |
|----------|------|
| `FunctionTemplateDecl` | 函数模板声明（未实例化） |
| `FunctionDecl`（含 `isInstantiation: true`） | 模板的显式实例化 |
| `TemplateTypeParmDecl` | 类型模板参数（`typename T`） |
| `NonTypeTemplateParmDecl` | 非类型模板参数（`size_t N`） |
| `ClassTemplateDecl` | 类模板声明（`Pair`） |

AST 片段示例（函数模板）：

```json
{
  "kind": "FunctionTemplateDecl",
  "name": "max_value",
  "inner": [
    { "kind": "TemplateTypeParmDecl", "name": "T" },
    {
      "kind": "FunctionDecl",
      "name": "max_value",
      "type": { "qualType": "T (T, T)" }
    }
  ]
}
```

## hicc 处理方式

### 模板函数的限制

**hicc 只支持已显式实例化的模板函数**。纯模板声明（`FunctionTemplateDecl`）无法直接生成 Rust 绑定，因为：
- 模板函数在编译时按具体类型展开
- Rust FFI 要求确定的 C ABI 函数签名

### 显式实例化 → 可绑定

在 C++ 代码中显式实例化后，可以通过具体类型进行绑定：

```cpp
// C++ 侧显式实例化
template int max_value<int>(int, int);
template double max_value<double>(double, double);
template void swap_values<int>(int&, int&);
```

对应的 Rust 绑定：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    // 显式实例化版本可以绑定
    #[cpp(func = "int max_value<int>(int, int)")]
    fn max_value_i32(a: i32, b: i32) -> i32;

    #[cpp(func = "double max_value<double>(double, double)")]
    fn max_value_f64(a: f64, b: f64) -> f64;

    #[cpp(func = "void swap_values<int>(int&, int&)")]
    fn swap_i32(a: &mut i32, b: &mut i32);
}
```

### 模板工厂函数（常用模式）

`std::make_unique` 是最常见的显式实例化模板函数场景：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    class MyClass;

    #[cpp(func = "std::unique_ptr<MyClass> std::make_unique<MyClass, int>(int&&)")]
    fn my_class_new(v: i32) -> MyClass;
}
```

### 模板结构体映射

对于 `Pair<T, U>` 等模板结构体，可以为特定实例化映射：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    // Pair<int, double> 的特定实例化
    class IntDoublePair = hicc_std::pair<hicc::Pod<i32>, hicc::Pod<f64>>;
}
```

### `convert<T, U>` 函数

```rust
hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "double convert<int, double>(int)")]
    fn int_to_double(v: i32) -> f64;
}
```

## 注意事项

1. **深度元编程不展开**：SFINAE、`enable_if`、`constexpr if` 等编译期逻辑不会体现在 Rust 绑定中，只能绑定最终实例化的具体版本
2. **链接符号**：模板函数的链接符号包含完整的类型信息（mangled name），需确保 C++ 侧有对应的显式实例化或显式特化
3. **`auto` 返回类型**：`auto` 返回类型在 AST 中会被推导为具体类型，hicc 使用推导后的类型
4. **可变参数模板**：`template<typename... Args>` 在当前版本中不直接支持，需手动展开为具体实例
