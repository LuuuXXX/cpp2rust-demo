# 示例 08：类模板

## 特性概述

本示例展示 C++ 的**类模板**，包括单类型参数类模板、多类型参数类模板、非类型模板参数以及模板类的方法。类模板是实现泛型数据结构的核心机制，hicc 通过类型别名和 `hicc_std` 的模式支持类模板映射。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| 单类型参数 | `template<typename T> class Stack` |
| 非类型参数 | `template<typename T, size_t SIZE = 100>` |
| 多类型参数 | `template<typename K, typename V> class Pair` |
| 缺省模板参数 | `size_t SIZE = 100` |
| 模板方法 | 访问泛型成员的方法 |

### 代码结构

```cpp
// 带非类型参数的栈模板
template<typename T, size_t SIZE = 100>
class Stack {
    void push(T value);
    T pop();
    bool is_empty() const;
    size_t size() const;
};

// 键值对模板
template<typename K, typename V>
class Pair {
    K get_key() const;
    V get_value() const;
    void set_key(const K& k);
    void set_value(const V& v);
};

// 固定数组模板
template<typename T, size_t N>
class FixedArray { ... };
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 节点：

| AST 节点 | 含义 |
|----------|------|
| `ClassTemplateDecl` | 类模板声明（含参数列表） |
| `TemplateTypeParmDecl` | 类型模板参数（`typename T`） |
| `NonTypeTemplateParmDecl` | 非类型模板参数（`size_t SIZE`） |
| `ClassTemplateSpecializationDecl` | 模板特化（显式实例化） |

AST 片段示例（类模板）：

```json
{
  "kind": "ClassTemplateDecl",
  "name": "Stack",
  "inner": [
    { "kind": "TemplateTypeParmDecl", "name": "T" },
    { "kind": "NonTypeTemplateParmDecl", "name": "SIZE", "type": { "qualType": "size_t" } },
    { "kind": "CXXRecordDecl", "name": "Stack", "inner": [...] }
  ]
}
```

## hicc 处理方式

### 类模板映射方式

hicc 对类模板的支持分为两种场景：

#### 方式一：为具体实例化映射（推荐用于业务代码）

为特定的模板实例化生成绑定：

```rust
// 方式一：使用 class alias 定义具体实例
hicc::import_lib! {
    #![link_name = "example"]

    // Stack<int, 50> 的实例
    class IntStack50 = Stack<hicc::Pod<i32>>;

    #[cpp(func = "Stack<int,50> stack_int_new()")]
    fn int_stack_new() -> IntStack50;
}

hicc::import_class! {
    #[cpp(class = "Stack<int, 50>")]
    class IntStack50 {
        #[cpp(method = "void push(int)")]
        fn push(&mut self, v: i32);

        #[cpp(method = "int pop()")]
        fn pop(&mut self) -> i32;

        #[cpp(method = "bool is_empty() const")]
        fn is_empty(&self) -> bool;

        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;
    }
}
```

#### 方式二：泛型模板类映射（适用于库封装）

参考 `hicc-std` 的模式，将 C++ 模板类映射为 Rust 泛型 struct：

```rust
hicc::import_class! {
    // 声明 C++ 完整模板签名
    #[cpp(class = "template<class T, size_t SIZE> Stack<T, SIZE>")]
    pub class Stack<T> {
        #[cpp(method = "bool is_empty() const")]
        pub fn is_empty(&self) -> bool;

        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        // 泛型方法：参数类型 T 自动转换为 AbiType::Output
        #[cpp(method = "void push(T)")]
        pub fn push(&mut self, v: T::Output);

        #[cpp(method = "T pop()")]
        pub fn pop(&mut self) -> T::Output;
    }
}

// 使用具体类型实例化
type IntStack = Stack<hicc::Pod<i32>>;
type FloatStack = Stack<hicc::Pod<f64>>;
```

### `Pair<K, V>` 模板类

```rust
hicc::import_class! {
    #[cpp(class = "template<class K, class V> Pair<K, V>")]
    pub class Pair<K, V> {
        #[cpp(method = "K get_key() const")]
        pub fn get_key(&self) -> K::Output;

        #[cpp(method = "V get_value() const")]
        pub fn get_value(&self) -> V::Output;
    }
}

type IntStrPair = Pair<hicc::Pod<i32>, hicc_std::string>;
```

### `hicc::Pod<T>` 用于 POD 类型

当模板参数是基础数值类型（`int`、`double` 等）时，需要用 `hicc::Pod<T>` 包装：

```rust
type IntStack = Stack<hicc::Pod<i32>>;    // Stack<int>
type DoubleStack = Stack<hicc::Pod<f64>>; // Stack<double>
type StringStack = Stack<hicc_std::string>; // Stack<std::string>（C++类直接使用）
```

## 注意事项

1. **显式实例化要求**：Rust 侧使用特定类型实例化时，C++ 侧必须有对应的显式实例化或定义（模板在头文件中）
2. **`AbiType` 约束**：泛型类的所有类型参数必须实现 `AbiType` trait，`hicc::Pod<T>` 为 POD 类型提供此实现
3. **非类型参数**：非类型模板参数（`size_t N`）无法映射为 Rust 泛型参数，通常需要固定为具体值
4. **模板成员函数**：模板类的成员方法签名中的类型需与实例化类型对应
