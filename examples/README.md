# C++ 特性示例与 hicc 脚手架

本目录包含全面的 C++ 特性示例，演示如何使用 hicc 实现 Rust-C++ FFI（外部函数接口）。

## 示例概览

| # | 示例 | C++ 特性 | Rust 模式 |
|---|---------|-------------|-------------|
| 01 | [basic-types](01-basic-types/) | 基础类型、函数、枚举、数组、指针 | `hicc::import_lib!` |
| 02 | [pointers-references](02-pointers-references/) | 指针、引用、函数指针、对象生命周期 | `ClassMutPtr`, `ClassRef` |
| 03 | [classes-basic](03-classes-basic/) | 类、构造函数、析构函数、访问控制、嵌套类 | `hicc::import_class!` |
| 04 | [inheritance](04-inheritance/) | 单继承/多继承、虚继承、访问说明符 | `#[interface]`, 多基类 |
| 05 | [virtual-polymorphism](05-virtual-polymorphism/) | 虚函数、纯虚函数、override、工厂模式 | 通过 `Interface` 实现动态分派 |
| 06 | [operator-overload](06-operator-overload/) | 运算符重载、转换运算符、下标运算符 | 方法包装的运算符 |
| 07 | [templates-function](07-templates-function/) | 函数模板、可变参数模板、模板特化 | 显式实例化 |
| 08 | [templates-class](08-templates-class/) | 类模板、模板参数、非类型参数 | 泛型语法 |
| 09 | [namespaces](09-namespaces/) | 命名空间、嵌套命名空间、内联命名空间 | `Namespace::function` |
| 10 | [stl-containers](10-stl-containers/) | std::vector, std::map, std::set, std::deque, std::pair | 使用 `hicc-std` 包装器 |
| 11 | [smart-pointers](11-smart-pointers/) | std::unique_ptr, std::shared_ptr, 自定义删除器 | `hicc::unique_ptr<T>` |
| 12 | [move-semantics](12-move-semantics/) | 移动构造函数、移动赋值运算符、右值引用 | 通过 FFI 实现移动语义 |
| 13 | [lambdas-functional](13-lambdas-functional/) | Lambda 表达式、std::function、闭包、捕获组 | `hicc::Function<fn()>` |
| 14 | [type-casting](14-type-casting/) | static_cast, dynamic_cast, const_cast, reinterpret_cast | 通过 FFI 进行类型转换 |
| 15 | [exceptions](15-exceptions/) | try/catch, throw, 自定义异常, noexcept | `hicc::Exception<T>` |
| 16 | [static-members](16-static-members/) | 静态成员变量、静态成员函数、单例模式 | `Class::static_method()` |
| 17 | [friend-functions](17-friend-functions/) | 友元函数、友元类、运算符友元重载 | 通过包装器访问友元 |
| 18 | [const-correctness](18-const-correctness/) | 常量方法、常量引用、可变成员、重载解析 | `const` 方法区分 |
| 19 | [memory-management](19-memory-management/) | new/delete, placement new, RAII, 对齐分配 | 通过 FFI 管理内存 |
| 20 | [template-specialization](20-template-specialization/) | 完全特化/偏特化、SFINAE、可变参数模板 | 模板特化 |

## 项目结构

每个示例遵循以下结构：

```
example-XX/
├── main.cpp           # 演示该特性的 C++ 源代码
├── ast.json           # Clang AST 的 JSON 格式导出（由 clang++ -ast-dump=json 生成）
├── Makefile           # C++ 编译配置
├── README.md          # 特性说明与 hicc 处理方式详细介绍
└── rust/              # （可选）对应的 Rust FFI 脚手架
    ├── Cargo.toml
    ├── build.rs
    └── src/
        └── main.rs
```

## 构建示例

### 构建 C++ 库

```bash
cd examples/01-basic-types
make
```

### 构建 Rust 项目

```bash
cd examples/01-basic-types/rust
cargo build
```

### 生成 AST JSON

```bash
cd examples/01-basic-types
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

## 涵盖的 C++ 特性

### 基础类型
- 整数类型（int, long, short, char）
- 浮点数类型（float, double）
- 布尔类型
- 指针和引用

### 类与对象
- 构造函数和析构函数
- 访问控制（public, private, protected）
- 静态成员
- 友元函数/类
- 常量正确性
- 移动语义

### 继承
- 单继承
- 多继承
- 虚继承（菱形继承问题）
- 多态（虚函数）

### 模板
- 函数模板
- 类模板
- 模板特化（完全特化和偏特化）
- 可变参数模板
- SFINAE

### 运算符
- 算术运算符
- 比较运算符
- 下标运算符
- 函数调用运算符
- 转换运算符

### 现代 C++
- 智能指针（unique_ptr, shared_ptr）
- Lambda 表达式
- std::function
- 异常处理
- 命名空间

### 内存管理
- new/delete
- Placement new
- RAII 惯用法
- 对齐分配

## hicc 最佳实践

1. **使用 `hicc::cpp!{}` 内联 C++ 定义** - 直接在 Rust 中嵌入 C++ 代码进行即时测试

2. **使用 `hicc::import_lib!{}` 声明外部库** - 使用 `#[cpp(func = "signature")]` 声明全局函数，使用 `class Foo;` 声明类前向声明

3. **使用 `hicc::import_class!{}` 定义 C++ 类** - 使用 `#[cpp(class = "Foo")]` 定义类，使用 `#[cpp(method = "signature")]` 包装类方法

4. **配合使用 `import_lib!` + `import_class!`** - `import_lib!` 声明工厂函数返回对象，`import_class!` 定义对象的方法

5. **使用 `#[interface]` 处理抽象类** - 启用基于接口的多态

6. **使用 `@make_proxy` 在 Rust 中实现 C++ 接口** - 允许 Rust 类型实现 C++ 接口

7. **使用 `hicc::Exception<T>` 处理异常** - 在 Rust 中捕获 C++ 异常

8. **使用 `hicc::Function<fn(T) -> U>` 传递回调** - 将 Rust 闭包传递给 C++

## AST 生成

`cpp-hook/` 目录包含一个 `LD_PRELOAD` 钩子，用于拦截 C++ 编译并自动生成 AST JSON 文件。

**注意**：此 hook 与 `c2rust-demo` 的 hook 是不同的：
- `c2rust-demo/hook/`：用于捕获 C 构建过程，生成 `.c2rust` 文件
- `examples/cpp-hook/`：用于拦截 C++ 编译，生成 Clang AST JSON

```bash
cd cpp-hook
make

# 使用编译
C2RUST_PROJECT_ROOT=$(pwd) \
C2RUST_FEATURE_ROOT=/tmp/ast \
LD_PRELOAD=libhook.so \
    g++ -c your_code.cpp
```

## 另请参阅

- [hicc 使用文档](../docs/usage-hicc.md)
- [c2rust-demo 使用文档](../docs/usage-c2rust-demo.md)
- [hicc 项目主页](https://github.com/DBJ2rics/hicc)
