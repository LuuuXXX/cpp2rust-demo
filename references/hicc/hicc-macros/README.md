# hicc-macros

## 简介

`hicc-macros` 是 hicc 生态中的**过程宏（procedural macro）库**，提供 `hicc::import_lib!`、`hicc::import_class!` 和 `hicc::cpp!` 三个核心宏的编译期展开实现。它基于 `hicc-autogen` 的代码生成能力，在 Rust 编译阶段将 hicc DSL 转换为合法的 Rust 代码。

## 提供的宏

### `import_lib!`

将 C++ 全局函数/变量接口声明转换为 Rust 绑定：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    // 类前向声明（告知宏哪些类型是 C++ 类）
    class MyClass;

    // 全局函数绑定
    #[cpp(func = "int add(int, int)")]
    fn add(a: i32, b: i32) -> i32;

    // 模板函数实例化
    #[cpp(func = "std::unique_ptr<MyClass> std::make_unique<MyClass, int>(int&&)")]
    fn my_class_new(v: i32) -> MyClass;

    // 静态成员函数
    #[cpp(func = "int Counter::get_total()")]
    fn counter_total() -> i32;

    // 带异常捕获的函数
    #[cpp(func = "int divide(int, int)")]
    fn divide(a: i32, b: i32) -> hicc::Exception<i32>;
}
```

**宏展开输出**：生成 Rust 外部函数声明（`extern "C"` 或 `extern "C++"` 风格）以及运行时安全包装器。

### `import_class!`

将 C++ 类定义转换为 Rust struct 及其方法实现：

```rust
hicc::import_class! {
    // 普通类
    #[cpp(class = "MyClass")]
    class MyClass {
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;

        #[cpp(method = "void set(int)")]
        fn set(&mut self, v: i32);
    }

    // 抽象接口类
    #[interface]
    class Shape {
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;
    }

    // 模板类（泛型）
    #[cpp(class = "template<class T> Container<T>")]
    pub class Container<T> {
        #[cpp(method = "bool empty() const")]
        pub fn empty(&self) -> bool;
    }

    // 类别名（引用外部定义的类）
    class StringAlias = hicc_std::string;
}
```

**宏展开输出**：
- 生成 Rust struct 定义（内含 `methods` vtable 指针和 `obj` 裸指针）
- 生成方法的 `impl` 块
- 对 `#[interface]` 类生成 Rust trait
- 实现 `AbiClass` trait

### `cpp!`

在 Rust 文件中嵌入 C++ 代码块（仅供 `hicc-build` 在构建时提取，运行时不展开任何内容）：

```rust
hicc::cpp! {
    #include <iostream>
    #include <string>

    static void hello_world() {
        std::cout << "Hello from C++!" << std::endl;
    }
}
```

**宏展开输出**：展开为空（`quote!()`），不生成任何 Rust 代码。C++ 内容由 `hicc-build` 在构建时提取并编译。

## 工作原理

```text
Rust 源文件
    │
    ├─ import_lib!{...}
    ├─ import_class!{...}       ──► hicc-macros（过程宏）
    └─ cpp!{...}                         │
                                         ▼
                              hicc-autogen（代码生成）
                                         │
                              ┌──────────┴──────────┐
                              │                     │
                        编译期（宏展开）         构建时（build.rs）
                              │                     │
                         Rust 绑定代码         C++ 适配代码（.cpp）
                                                     │
                                              cc::Build 编译
                                                     │
                                              静态库（.a）
```

## 与 `hicc-autogen` 的关系

| 功能 | `hicc-macros` | `hicc-autogen` |
|------|--------------|----------------|
| 触发时机 | Rust 编译器展开宏时 | `build.rs` 执行时 |
| 输入 | `proc_macro::TokenStream` | Rust 源文件路径 |
| 输出（`import_lib!`） | Rust 绑定代码 | C++ 适配代码字符串 |
| 输出（`cpp!`） | 空（`quote!()`） | C++ 代码块提取 |

两者共享 `hicc-autogen` 的解析逻辑（`ImportLib`、`ImportClass`、`Cpp` 的 `syn::parse` 实现）。

## 宏展开示例

```rust
// 输入
hicc::import_lib! {
    #![link_name = "example"]
    #[cpp(func = "int add(int, int)")]
    fn add(a: i32, b: i32) -> i32;
}

// 展开后的 Rust 代码（简化示意）
extern "C" {
    #[link_name = "__hicc_add_int_int"]
    fn __hicc_add(a: i32, b: i32) -> i32;
}

pub fn add(a: i32, b: i32) -> i32 {
    unsafe { __hicc_add(a, b) }
}
```

## Cargo.toml 配置

```toml
[dependencies]
hicc = "0.2"  # 包含 hicc-macros 作为依赖
```

由于 `hicc-macros` 是过程宏 crate（`proc-macro = true`），通常通过 `hicc` 主 crate 间接使用，不直接依赖。

## 注意事项

1. **过程宏调试**：可以使用 `cargo expand` 工具查看宏展开后的代码：
   ```bash
   cargo install cargo-expand
   cargo expand
   ```
2. **错误信息**：宏展开错误会以编译错误的形式报告，错误位置指向宏调用处
3. **顺序依赖**：`hicc::cpp!{}` 必须在 `import_lib!{}` 之前，因为 C++ 代码块在构建时按声明顺序提取
4. **proc-macro crate 限制**：过程宏 crate 只能导出宏，不能导出普通函数或类型；运行时类型定义在 `hicc` 主 crate 中
