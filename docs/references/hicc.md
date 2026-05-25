# hicc 功能详解

## 概述

**hicc** (Header-Included C++ Calling) 是一个 Rust-C++ 互操作框架，允许在 Rust 代码中直接内联 C++ 代码并与之互操作。与传统的 FFI 不同，hicc 通过过程宏和构建时编译实现了 Rust 与 C++ 之间的高度集成。

项目地址：`./references/hicc`（仅包含 examples，源代码为外部 crate）

## 核心概念

### 1. 过程宏驱动的 FFI

hicc 使用 Rust 过程宏来：
- 解析 C++ 代码片段
- 生成 Rust FFI bindings
- 在构建时编译 C++ 代码

### 2. 三个主要组件

从 examples 的 `Cargo.toml` 可以看出 hicc 包含多个 crate：

```toml
[dependencies]
hicc = { path = "../../hicc", version = "0.2" }
hicc-std = { path = "../../hicc-std", version = "0.2" }

[build-dependencies]
hicc-build = { path = "../../hicc-build", version = "0.2" }
```

- **hicc**：核心库，提供 `cpp!`、`import_class!`、`import_lib!` 等宏
- **hicc-build**：构建时依赖，负责编译内联的 C++ 代码
- **hicc-std**：C++ STL 容器的 Rust 包装实现

## 核心宏

### 1. hicc::cpp! 宏

`cpp!` 宏允许在 Rust 源文件中直接编写 C++ 代码：

```rust
hicc::cpp! {
    #include <iostream>
    static void hello_world() {
        std::cout << "hello world!" << std::endl;
    }
}
```

**特性**：
- C++ 代码在构建时被编译为机器码
- 支持 `#include` 任意头文件
- 支持 C++11/14/17 特性（模板、lambda、auto 等）
- 可以定义类、函数、全局变量
- 支持 `#pragma pack` 等预处理指令

### 2. hicc::import_lib! 宏

`import_lib!` 宏将 C++ 函数和类声明导入为 Rust：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    class string;

    #[cpp(func = "std::string foo(const std::string&)")]
    fn foo_const(name: &string) -> string;

    #[cpp(func = "std::unique_ptr<std::string> hicc::make_unique<std::string, const char*>(const char*&&)")]
    unsafe fn string_new(s: *const u8) -> string;

    #[cpp(func = "int printf(const char* , ...)")]
    unsafe fn printf(format: *const u8, ...) -> i32;
}
```

**属性**：
- `#[link_name = "..."]`：指定链接的库名
- `#[cpp(func = "...")]`：指定对应的 C++ 函数签名
- `class XXX;`：声明一个 C++ 类类型

### 3. hicc::import_class! 宏

`import_class!` 宏将 C++ 类导入为 Rust struct：

```rust
hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "int foo(int)")]
        fn foo_mut(&mut self, v: i32) -> i32;

        #[cpp(method = "int foo(int) const")]
        fn foo_const(&self, v: i32) -> i32;

        #[cpp(method = "int bar(int, int) const")]
        fn bar(&self, v: i32) -> hicc::Exception<()>;
    }
}
```

**支持的 C++ 特性**：
- 普通方法
- const 方法
- volatile 方法
- const volatile 方法
- 右值引用方法（`&&`）
- 默认参数
- 异常（通过 `hicc::Exception<T>`）
- 字段访问
- 静态数据成员

### 4. #[interface] 属性

用于将 C++ 抽象类映射为 Rust trait：

```rust
hicc::import_class! {
    #[interface]
    class Foo {
        #[cpp(method = "void foo() const")]
        fn foo(&self);
    }

    #[interface]
    class Bar: Foo {
        #[cpp(method = "void bar() const")]
        fn bar(&self);
    }

    #[cpp(class = "Baz", ctor = "Baz()")]
    class Baz: Bar {
        #[cpp(method = "void baz() const")]
        fn baz(&self);
    }
}
```

## 类型系统

### 1. 智能指针

hicc 提供了 C++ 智能指针的 Rust 包装：

```rust
// std::shared_ptr<T>
hicc::shared_ptr<T>

// std::unique_ptr<T>
hicc::unique_ptr<T>

// 自定义删除器
hicc::unique_ptr<T, D>
```

示例：
```rust
#[cpp(func = "std::shared_ptr<int> make_shared_int(int)")]
fn int_sptr(v: i32) -> hicc::shared_ptr<hicc::Pod<i32>>;

// 使用
let iptr = int_sptr(100);
println!("value = {}", unsafe { *iptr.get() });
let weak = iptr.weak();  // 创建 weak_ptr
```

### 2. 裸指针和引用

```rust
// 不可空指针
hicc::ClassRef<T>      // 对应 C++ const T&
hicc::ClassRefMut<T>   // 对应 C++ T&

// 可空指针
*const T
*mut T
```

### 3. ABI 类型

```rust
hicc::AbiClass<T>     // 用于 placement new 等场景
hicc::Pod<T>          // Plain Old Data，用于原始类型
hicc::Exception<T>    // C++ 异常包装，成功返回 Ok(T)，失败返回 Err
```

### 4. Function 类型

```rust
hicc::Function<fn(i32) -> i32>
```

用于 C++ `std::function<R(Args...)>` 的互操作：
```rust
#[cpp(func = "int foo(int, std::function<int(int)>)")]
fn foo(v: i32, func: hicc::Function<fn(i32) -> i32>) -> i32;

// 传递 Rust 闭包
let val = foo(10, |v: i32| -> i32 {
    println!("rust v = {v}");
    v + 10
}.into());
```

### 5. RustAny 类型

允许将任意 Rust 类型存储在 C++ 容器中：

```rust
hicc::RustAny<Key>           // 通用包装
hicc::RustKey<Key>          // 用于 std::map 键
hicc::RustHashKey<Key>       // 用于 std::unordered_map 键
```

示例：
```rust
#[derive(Clone, PartialEq, PartialOrd, Hash)]
struct Key {
    val: i32,
    key: i32,
}

let key = hicc::RustAny::new_clone(Key::new(1, 1));
vec.push_back(&key);
let back = vec.back().unwrap();
```

## 构建系统

### build.rs 模式

```rust
fn main() {
    hicc_build::Build::new()
        .rust_file("src/main.rs")  // 指定包含 cpp! 宏的 Rust 文件
        .compile("example");        // 编译为指定名称的库

    println!("cargo::rustc-link-lib=example");
    #[cfg(not(all(target_os = "windows", target_env = "msvc")))]
    println!("cargo::rustc-link-lib=stdc++");  // 链接 C++ 标准库

    println!("cargo::rerun-if-changed=src/main.rs");
}
```

**工作流程**：
1. `hicc-build` 解析 Rust 源文件中的 `cpp!` 宏
2. 提取内联的 C++ 代码
3. 调用 C++ 编译器编译为静态库
4. 生成链接指令

## 功能示例

### 1. Hello World

```rust
hicc::cpp! {
    #include <iostream>
    static void hello_world() {
        std::cout << "hello world!" << std::endl;
    }
}

hicc::import_lib! {
    #![link_name = "example"]
    #[cpp(func = "void hello_world()")]
    fn hello_world();
}

fn main() {
    hello_world();
}
```

### 2. C++ 类导入

```rust
hicc::cpp! {
    #include <iostream>
    class Foo {
    public:
        Foo() { std::cout << "Foo::Foo()" << std::endl; }
        ~Foo() { std::cout << "Foo::~Foo()" << std::endl; }
        int foo(int v) { return v; }
        int foo(int v) const { return v; }
    };
}

hicc::import_class! {
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "int foo(int)")]
        fn foo_mut(&mut self, v: i32) -> i32;
        #[cpp(method = "int foo(int) const")]
        fn foo_const(&self, v: i32) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "example"]
    class Foo;
    #[cpp(func = "std::unique_ptr<Foo> hicc::make_unique<Foo>()")]
    fn foo_new() -> Foo;
}

fn main() {
    let foo = foo_new();
    println!("foo return: {}", foo.foo_mut(1));
}
```

### 3. 接口与实现

hicc 支持 C++ 抽象类与 Rust trait 的双向映射：

```rust
// C++ 端定义抽象类
hicc::cpp! {
    struct Foo {
        virtual ~Foo() {};
        virtual void foo() const = 0;
    };
    struct Baz { virtual void baz() const = 0; };
}

// Rust 端实现 trait
struct RustBaz;

impl Foo for RustBaz {
    fn foo(&self) { println!("Rust Baz::foo"); }
}

impl Baz for RustBaz {
    fn baz(&self) { println!("Rust Baz::baz"); }
}

// Rust 实现可以作为 C++ 参数传递
#[cpp(func = "Baz @make_proxy<Baz>()")]
#[interface(name = "Bar")]
fn new_rust_baz(intf: hicc::Interface<Baz>) -> Baz;
```

### 4. STL 容器

hicc-std 提供了 C++ 容器的 Rust 包装：

```rust
hicc::cpp! {
    #include <hicc/std/map.hpp>
    #include <hicc/std/string.hpp>
    typedef std::map<int, std::string> CppMap;
}

hicc::import_lib! {
    #![link_name = "example"]
    class RustMap = hicc_std::map<hicc::Pod<i32>, hicc_std::string>;
    #[cpp(func = "std::unique_ptr<CppMap> hicc::make_unique<CppMap>()")]
    fn rustmap_new() -> RustMap;
}

fn main() {
    let mut map = rustmap_new();
    let name = hicc_std::string::from(c"hello");
    map.insert(&0, &name);
    assert_eq!(map.get(&0), Some(name.as_ref()));
}
```

### 5. 内存管理

```rust
// 自定义析构器
hicc::cpp! {
    class Foo {
    public:
        static Foo* new_instance() { return new Foo; }
        static void free_instance(Foo* foo) { delete foo; }
    };
}

hicc::import_class! {
    #[cpp(class = "Foo", destroy = "Foo::free_instance")]
    class Foo {
        fn new() -> Self { foo_new() }
    }
}

hicc::import_lib! {
    #[cpp(func = "Foo* Foo::new_instance()")]
    fn foo_new() -> Foo;
}

fn main() {
    let foo = Foo::new();
    let foo = unsafe { foo.into_unique() };  // 获取 unique ownership
    std::mem::drop(foo);  // 调用自定义析构器
}
```

### 6. Placement New

```rust
hicc::import_lib! {
    #[cpp(func = "hicc::AbiClass<std::string> hicc::placement_new<std::string, const char*>(void*, size_t, const char*&&)")]
    fn cpp_string_ctor(buf: *mut i8, len: usize, s: *const i8) -> &'static mut string;
}

fn main() {
    let mut buf = [1_i8; 100];
    let rs = cpp_string_ctor(buf.as_mut_ptr(), buf.len(), c"hello".as_ptr());
}
```

## hicc 与其他工具的对比

### hicc vs c2rust-demo

| 特性 | hicc | c2rust-demo |
|------|------|-------------|
| **原理** | 手动编写 C++ 代码片段 | 自动捕获构建过程 |
| **使用方式** | 在 Rust 中内联 C++ | 拦截编译器调用 |
| **输入** | 需要显式编写 C++ | 自动分析现有 C 代码 |
| **输出** | Rust FFI 调用 C++ | 生成 Rust 脚手架 |
| **自动化程度** | 手动 | 自动 |
| **适用场景** | 增量迁移、新写互操作 | 大规模 C→Rust 迁移 |

**结论**：hicc **不能替代** c2rust-demo。hicc 需要开发者手动编写 C++ 代码，而 c2rust-demo 可以自动从现有 C 代码生成 Rust FFI 层。

### hicc vs rapidjson_sys

| 特性 | hicc | rapidjson_sys |
|------|------|---------------|
| **代码组织** | 所有代码在一个 .rs 文件 | 分离 .h / .cpp / .rs |
| **宏能力** | 使用过程宏 | 使用 bindgen |
| **语法** | 更简洁直观 | 更显式 |
| **构建时生成** | cpp! 宏自动提取 | build.rs 中手动调用 bindgen |
| **功能** | 类/函数/智能指针/容器/接口 | 仅函数和 opaque handle |

**结论**：hicc **可以实现 rapidjson_sys 的功能**，且语法更简洁。

**使用 hicc 重写 rapidjson_sys 的示例**：

```rust
// rapidjson_sys 用 hicc 重写
hicc::cpp! {
    #include "rapidjson/internal/biginteger.h"
    using namespace rapidjson::internal;

    struct RapidJsonBigIntegerHandle {
        BigInteger value;
        RapidJsonBigIntegerHandle() : value(0) {}
    };

    RapidJsonBigIntegerHandle* rapidjson_biginteger_new() {
        return new (std::nothrow) RapidJsonBigIntegerHandle();
    }

    void rapidjson_biginteger_free(RapidJsonBigIntegerHandle* handle) {
        delete handle;
    }

    int rapidjson_biginteger_from_decimal_literal(
        RapidJsonBigIntegerHandle* handle,
        const char* literal
    ) {
        if (!handle || !literal) return 0;
        handle->value = BigInteger(literal, std::strlen(literal));
        return 1;
    }

    void rapidjson_biginteger_add_u64(
        RapidJsonBigIntegerHandle* handle,
        unsigned long long value
    ) {
        if (!handle) return;
        handle->value += static_cast<uint64_t>(value);
    }

    int rapidjson_biginteger_compare(
        const RapidJsonBigIntegerHandle* a,
        const RapidJsonBigIntegerHandle* b
    ) {
        if (!a || !b) return 0;
        return a->value.Compare(b->value);
    }
}

hicc::import_class! {
    #[cpp(class = "RapidJsonBigIntegerHandle", destroy = "rapidjson_biginteger_free")]
    class RapidJsonBigIntegerHandle {
        #[cpp(method = "int rapidjson_biginteger_from_decimal_literal(const char*)")]
        fn from_decimal_literal(&mut self, lit: *const i8) -> i32;

        #[cpp(method = "void rapidjson_biginteger_add_u64(unsigned long long)")]
        fn add_u64(&mut self, value: u64);

        #[cpp(method = "int rapidjson_biginteger_compare(const RapidJsonBigIntegerHandle*)")]
        fn compare(&self, other: &RapidJsonBigIntegerHandle) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "rapidjson_sys"]

    #[cpp(func = "RapidJsonBigIntegerHandle* rapidjson_biginteger_new()")]
    fn rapidjson_biginteger_new() -> RapidJsonBigIntegerHandle;
}
```

## 局限性

1. **编译时间**：每次修改 C++ 代码都需要重新编译
2. **错误信息**：C++ 编译错误可能难以调试
3. **调试困难**：C++ 和 Rust 之间的调用不易追踪
4. **平台依赖**：需要 C++ 编译器支持
5. **二进制大小**：静态链接 C++ 代码会增加最终二进制大小

## 适用场景

1. **增量迁移**：在现有 Rust 项目中逐步引入 C++ 逻辑
2. **性能关键代码**：对性能要求极高的部分使用 C++ 实现
3. **复用现有 C++ 库**：无需重写现有 C++ 代码
4. **FFI 简化**：比传统 bindgen +手动编写更简洁
5. **接口测试**：通过 C++ 实现验证 Rust 算法正确性（如 L1/L2 测试）

## 构建要求

- Rust 编译器（支持过程宏）
- C++ 编译器（g++ 或 clang++）
- Cargo 和 Rust 工具链

## 文件结构（hicc/examples）

```
hicc/examples/
├── hello-world/       # 基础 hello world
├── functions/         # 函数导入
├── class/             # 类导入
├── datas/             # 字段和静态数据
├── destroy/           # 自定义析构器
├── dynamic_cast/      # 运行时类型识别
├── functional/        # std::function 互操作
├── interface/         # 抽象类与 Rust trait
├── memory/            # 智能指针
├── placement_new/     # Placement new
├── rust_any/          # 任意 Rust 类型存储
├── stl/               # STL 容器
└── hicc-std/          # hicc-std 示例
```

## 总结

**hicc** 是一个强大的 Rust-C++ 互操作框架，它通过过程宏和构建时编译实现了：

1. **内联 C++ 代码**：无需分离的 .h/.cpp 文件
2. **类型安全包装**：C++ 类映射为 Rust struct
3. **智能指针互操作**：shared_ptr/unique_ptr 的 Rust 包装
4. **接口映射**：C++ 抽象类 ↔ Rust trait
5. **容器支持**：STL 容器的 Rust 包装
6. **闭包互操作**：Rust 闭包传递给 C++

**与 c2rust-demo 的关系**：两者解决不同问题，hicc 不能替代 c2rust-demo 的自动构建捕获功能。

**与 rapidjson_sys 的关系**：hicc 可以用更简洁的语法实现相同的 FFI 功能。
