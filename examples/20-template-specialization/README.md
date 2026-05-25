# 示例 20：模板特化

## 特性概述

本示例展示 C++ 的**模板特化**，包括完全特化（full specialization）、偏特化（partial specialization）、SFINAE（Substitution Failure Is Not An Error）以及类型萃取（type traits）。cpp2rust-demo 通过识别 AST 中的 `ClassTemplateSpecializationDecl` 节点来处理显式特化。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| 完全特化 | `template<> class Wrapper<int>` |
| 偏特化 | `template<typename T> class Wrapper<T*>` |
| SFINAE | `std::enable_if_t` 编译期条件 |
| 类型萃取 | `std::is_same`、`std::is_pointer` 等 |
| 函数模板特化 | `template<> T max<T>(T, T)` |

### 代码结构

```cpp
// 主模板
template<typename T>
class Wrapper {
    T get() const;
};

// 完全特化（int）
template<>
class Wrapper<int> {
    int get() const;
    int get_squared() const;  // int 特有方法
};

// 偏特化（指针类型）
template<typename T>
class Wrapper<T*> {
    T* get() const;
    bool is_null() const;     // 指针特有方法
};

// 偏特化（const 类型）
template<typename T>
class Wrapper<const T> { ... };
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 节点：

| AST 节点 | 含义 |
|----------|------|
| `ClassTemplateDecl` | 主模板声明 |
| `ClassTemplateSpecializationDecl` | 完全/偏特化声明 |
| `ClassTemplatePartialSpecializationDecl` | 偏特化 |
| `FunctionTemplateDecl` | 函数模板 |

AST 片段示例（完全特化）：

```json
{
  "kind": "ClassTemplateSpecializationDecl",
  "name": "Wrapper",
  "templateArgs": [
    { "type": { "qualType": "int" } }
  ],
  "inner": [
    { "kind": "CXXMethodDecl", "name": "get", "type": { "qualType": "int () const" } },
    { "kind": "CXXMethodDecl", "name": "get_squared", "type": { "qualType": "int () const" } }
  ]
}
```

## hicc 处理方式

### 完全特化 → 独立类映射

每个完全特化实例可以独立映射，因为它们有确定的方法集合：

```rust
// 主模板的 Wrapper<double> 实例
hicc::import_class! {
    #[cpp(class = "Wrapper<double>")]
    class WrapperDouble {
        #[cpp(method = "double get() const")]
        fn get(&self) -> f64;
    }
}

// 完全特化 Wrapper<int> —— 有额外的 get_squared() 方法
hicc::import_class! {
    #[cpp(class = "Wrapper<int>")]
    class WrapperInt {
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;

        #[cpp(method = "int get_squared() const")]
        fn get_squared(&self) -> i32;
    }
}

// 偏特化 Wrapper<T*> —— 有指针特有方法
hicc::import_class! {
    #[cpp(class = "Wrapper<int*>")]
    class WrapperIntPtr {
        #[cpp(method = "int* get() const")]
        fn get(&self) -> *mut i32;

        #[cpp(method = "bool is_null() const")]
        fn is_null(&self) -> bool;
    }
}
```

### 工厂函数

各特化实例需要单独的工厂函数：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    class WrapperInt;
    class WrapperDouble;

    #[cpp(func = "std::unique_ptr<Wrapper<int>> std::make_unique<Wrapper<int>, int>(int&&)")]
    fn wrapper_int_new(v: i32) -> WrapperInt;

    #[cpp(func = "std::unique_ptr<Wrapper<double>> std::make_unique<Wrapper<double>, double>(double&&)")]
    fn wrapper_double_new(v: f64) -> WrapperDouble;
}
```

### 类型别名映射

使用类型别名可以简化特化实例的命名：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    // 通过别名定义特化实例的 Rust 类型
    class WrapperInt = Wrapper<hicc::Pod<i32>>;
    class WrapperStr = Wrapper<hicc_std::string>;
}
```

### SFINAE 的处理

SFINAE 本质上是编译期条件选择，在 AST 中只会保留满足条件的实例化结果。对于 Rust FFI，只需关注最终被实例化的具体类型：

```rust
// SFINAE 的结果：只有 T 满足约束的版本才出现在 AST 中
// 例如 enable_if<std::is_integral<T>::value>
// → Rust 侧只需要绑定具体的 int、long、short 版本

hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "int sfinae_add<int>(int, int)")]
    fn sfinae_add_i32(a: i32, b: i32) -> i32;
}
```

### AliasRegistry（类型别名注册）

cpp2rust-demo 内部通过 `AliasRegistry` 管理模板类型别名：
- `template_to_alias`：模板名 → 别名列表
- `type_to_alias`：完整 C++ 类型 → 第一个别名

当遇到 `Wrapper<int>` 时，会通过 `AliasRegistry` 查找对应的 Rust 类型名称。

## 注意事项

1. **显式实例化要求**：偏特化/完全特化在 Rust 侧绑定前必须在 C++ 侧有显式实例化或定义在头文件中
2. **偏特化不直接映射到 Rust 泛型**：Rust 不支持类似 C++ 的模板偏特化，每种具体类型组合需要单独绑定
3. **SFINAE 对 FFI 透明**：SFINAE 在编译期完成，Rust FFI 只看到最终实例化的具体函数/类，不需要处理 SFINAE 本身
4. **`type_traits` 使用**：`std::is_same`、`std::enable_if` 等 type traits 是编译期元编程，不产生运行时代码，Rust 侧不需要映射
5. **可变参数特化**：可变参数模板特化（variadic template specialization）在当前版本中不完全支持
