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
    pub class Foo {
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

### 2.1 class-in-lib 语法（class body）

`import_lib!` 中的 `class` 声明支持带函数体的形式，可在宏内部直接定义关联函数（无 `self`）和方法（有 `self`）：

- **关联函数（无 `self`）**：自动提取为 `#[member]` 全局函数，生成 `impl ClassName { fn ... }`。
- **方法（有 `self`）**：保留在类中，生成 `import_class!` 调用。

```rust
hicc::import_lib! {
    #![link_name = "example"]

    class Foo;
    class Generic<T>;

    // 非泛型类：关联函数（无 self）被提取到 impl 块
    #[cpp(class = "Foo")]
    class Foo {
        #[cpp(method = "void bar() const")]
        fn bar(&self);

        // 无 self → 生成 impl Foo { fn new() -> Foo }
        #[cpp(func = "Foo* Foo::new_instance()")]
        fn new() -> Foo;
    }

    // 泛型类：关联函数不能使用类的泛型参数，必须使用具体类型
    #[cpp(class = "Generic")]
    class Generic<T> {
        #[cpp(method = "void display() const")]
        fn display(&self);

        #[cpp(func = "Generic<int>* hicc_new_generic_int()")]
        fn new() -> Generic<hicc::Pod<i32>>;
    }
}

fn main() {
    let foo = Foo::new();          // 通过 impl Foo 调用
    let gen = Generic::<hicc::Pod<i32>>::new();
}
```

**注意事项**：
1. 关联函数必须使用 `#[cpp(func = "...")]` 标注实际 C++ 函数。
2. 泛型类的关联函数中不能使用类的泛型参数，必须使用具体类型。
3. 参见 `references/hicc/examples/import_lib_class` 完整示例。

### 4. #[interface] 属性

用于将 C++ 抽象类映射为 Rust trait：

```rust
hicc::import_class! {
    #[interface]
    pub class Foo {
        #[cpp(method = "void foo() const")]
        fn foo(&self);
    }

    #[interface]
    pub class Bar: Foo {
        #[cpp(method = "void bar() const")]
        fn bar(&self);
    }

    #[cpp(class = "Baz", ctor = "Baz()")]
    pub class Baz: Bar {
        #[cpp(method = "void baz() const")]
        fn baz(&self);
    }
}
```

### 5. #[member] 属性

`#[member]` 用于将 `import_lib!` 中的独立函数绑定为某个已定义类型的关联函数：

```rust
struct MyMap;

hicc::import_lib! {
    #![link_name = "example"]

    #[member(class = "MyMap", method = "new")]
    #[cpp(func = "std::unique_ptr<std::map<int, int>> hicc::make_unique<std::map<int, int>>()")]
    fn mapintint_new();
}

fn main() {
    let mut map = MyMap::new();  // 通过 impl MyMap 调用
}
```

**说明**：
- `class`：目标类型名（当前 crate 中已定义的 struct）。
- `method`：生成的关联函数名。
- 与 class-in-lib 语法不同，`#[member]` 手动控制绑定关系，适用于需要将已有函数绑定到外部类型的场景。
- class-in-lib 语法（无 `self` 的关联函数）会自动添加 `#[member]` 完成相同效果。

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
    pub class Foo {
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
    pub class Foo {
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

## cpp2rust-demo 的模板映射（v6）

> 本节说明 cpp2rust-demo 工具如何把 C++ 模板声明映射为 hicc 骨架，对应 hicc 的模板类 / 模板函数 / 实例化能力。该能力由环境变量 `CPP2RUST_GEN_TEMPLATES` 控制，**默认关闭**（设为 `1`/`true`/`yes`/`on` 开启），关闭时默认产物逐字节不变。所有骨架均带 `cpp2rust-todo[TMPL]` 占位注释，提示用户按实际实例化类型校验签名与 `AbiType` 约束后补全。

### 1. 模板类 → 泛型 `import_class!` 骨架

C++ 模板类（`template<typename T> class Stack { ... }`）映射为泛型 `pub class Stack<T>`，成员方法签名中保留泛型 `T`：

```rust
hicc::import_class! {
    // cpp2rust-todo[TMPL]: 模板类泛型骨架，请按实际实例化类型校验签名与 AbiType 约束；
    #[cpp(class = "template<class T> Stack<T>")]
    pub class Stack<T> {
        #[cpp(method = "void push(T value)")]
        pub fn push(&mut self, value: T);
        #[cpp(method = "T top() const")]
        pub fn top(&self) -> T;
    }
}
```

### 2. 模板实例化别名

工具扫描当前编译单元中「以具体类型实例化本文件模板类」的使用点（字段类型、方法参数 / 返回类型、全局函数参数 / 返回类型），生成 hicc 形式的类型别名。POD 标量用 `hicc::Pod<...>` 包装，命名风格与 hicc `reference.md` 一致：

```rust
pub type StackI32 = Stack<hicc::Pod<i32>>;
pub type StackF64 = Stack<hicc::Pod<f64>>;
```

类类型实参保留原 C++ 名并附 TODO，提示用户替换为对应的 hicc 类（如 `hicc_std::string`）。

### 3. 构造工厂骨架

由模板类公有构造函数派生工厂函数，将类型参数 `T` 替换为实例化的具体类型，在 `import_lib!` 中输出：

```rust
hicc::import_lib! {
    // cpp2rust-todo[TMPL]: StackI32 构造工厂骨架 —— 需在 C++ 侧提供对应符号并校验签名
    #[cpp(func = "Stack<int>* stack_i32_new(int initial)")]
    pub unsafe fn stack_i32_new(initial: i32) -> StackI32;
}
```

工厂对应的 C++ 符号通常需用户在 C++ 侧显式实例化 / 包装后才存在，故标记为 TODO。

### 4. 模板函数 → 泛型 `import_lib!` 骨架

C++ 模板函数（`template<typename T> void do_swap(T*, T*)`）映射为泛型 `import_lib!` 绑定，`<T>` 为占位，需替换为实际实例化类型：

```rust
hicc::import_lib! {
    // cpp2rust-todo[TMPL]: 模板函数需按实例化类型声明（如 do_swap<int>(int*, int*)）；
    #[cpp(func = "void do_swap<T>(T*, T*)")]
    pub unsafe fn do_swap(a: *mut T, b: *mut T);
}
```

## cpp2rust-demo 的接口代理映射（v6 Phase C）

> 本节说明 cpp2rust-demo 工具如何把「继承 C++ 抽象接口的具体类」映射为 hicc 的
> `@make_proxy` 代理工厂骨架，使 Rust 侧可实现 C++ 抽象类。该能力由环境变量
> `CPP2RUST_GEN_PROXY` 控制，**默认关闭**（设为 `1`/`true`/`yes`/`on` 开启），关闭时默认
> 产物逐字节不变。骨架带 `cpp2rust-todo[PROXY]` 占位注释，提示用户提供接口实现并校验
> `@make_proxy` 参数类型列表。

纯虚接口类已映射为 `#[interface]`（参与默认产物）。在此基础上，对「非抽象、可实例化，
且继承自某个纯虚接口基类」的具体类，工具由其公有构造函数（排除拷贝 / 移动构造）派生
`@make_proxy` 工厂，结合直接接口基类的 `#[interface(name = ...)]`：

```rust
hicc::import_lib! {
    // cpp2rust-todo[PROXY]: @make_proxy 工厂骨架 —— 使 Rust 侧可实现 C++ 接口 Bar；
    // 需确认构造函数参数类型列表与 @make_proxy 一致，Rust 实现类经 hicc::Interface<Baz> 传入。
    #[cpp(func = "Baz @make_proxy<Baz>()")]
    #[interface(name = "Bar")]
    fn new_rust_baz(intf: hicc::Interface<Baz>) -> Baz;
}
```

第一个参数固定为 Rust 实现类（`hicc::Interface<具体类>`），其后接构造函数参数。

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
    pub class RapidJsonBigIntegerHandle {
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
7. **class-in-lib 语法**：在 import_lib! 内直接定义类方法与关联函数
8. **Rust→C 导出（hicc-rs）**：通过过程宏将 Rust 类型导出为 C++ 可调用接口

**与 c2rust-demo 的关系**：两者解决不同问题，hicc 不能替代 c2rust-demo 的自动构建捕获功能。

**与 rapidjson_sys 的关系**：hicc 可以用更简洁的语法实现相同的 FFI 功能。

## hicc-rs 子项目（Rust→C 导出）

`hicc-rs` 是 hicc 项目中专门用于 **Rust→C 导出**的子 crate，提供过程宏将 Rust 类型的方法包装为 C 兼容的 FFI 接口。

### 三个导出宏

| 宏 | 作用 | 适用场景 |
|----|------|---------|
| `#[export_class]` | 将 Rust `impl` 块方法包装为 C 函数 | 导出面向对象的接口 |
| `#[export_lib]` | 将 Rust 全局函数包装为 C 函数库 | 导出过程式函数库 |
| `#[export_pod]` | 为 POD 类型生成 `TypeName` trait 实现 | 注册纯数据类型的 C++ 名称 |

### export_class 用法

```rust
use hicc_rs::export_class;

#[export_class]
impl MyType {
    fn get_value(&self) -> i32;
    fn set_value(&mut self, v: i32);
}
```

生成内容：`ValueType` 实现 + 包装结构体（`MyTypeClass`）+ C 适配函数 + `MyTypeMethods` 结构体。

**特性**：
- 支持泛型参数（`impl<T> MyContainer<T>`）
- 支持生命周期参数（`impl<'a> Slice<'a, T>`）
- 支持常量泛型（`impl<T, const N: usize> Array<T, N>`）
- 支持深度分组（引用/指针嵌套深度最多 4 层）
- `in_hicc` 属性：`#[export_class(in_hicc)]`，将 `::hicc_rs::` 替换为 `crate::`（用于 hicc 内部）

### export_lib 用法

```rust
use hicc_rs::export_lib;

#[export_lib(export_name = "get_ffi")]
mod ffi {
    fn my_function(val: &Option<i32>) -> bool;
}
```

**特性**：
- 无方法体：适配函数通过 `crate::function_name()` 调用外部实现
- 有方法体：适配函数直接使用方法体中的代码
- `in_hicc` 属性：同 `export_class`

### export_pod 用法

```rust
use hicc_rs::export_pod;

#[export_pod]
type MyPod;

// 带泛型参数
#[export_pod]
type MyPod<'a, T: Bound>;
```

作用：为该类型生成 `TypeName` trait 实现，注册其 C++ 类型名称，使其可用于 hicc 类型映射。

### 使用示例（来自 hicc 内部）

```rust
use crate::export_class;

#[export_class(in_hicc)]
impl<T> Option<T> {
    fn is_some(&self) -> bool;
    fn is_none(&self) -> bool;
    fn as_ptr(&self) -> *const T;
}

#[export_class(in_hicc)]
impl<T, const N: usize> Array<T, N> {
    fn len(&self) -> usize { N }
    fn get(&self, idx: usize) -> &T;
    fn set(&mut self, idx: usize, val: T);
}
```

### 文件结构（hicc-rs）

```
hicc-rs/
├── src/
│   ├── core/
│   │   ├── option.rs     # Option<T>
│   │   ├── array.rs      # Array<T, N>
│   │   ├── slice.rs      # Slice<'a, T> / SliceMut<'a, T>
│   │   ├── result.rs     # Result<T, E>
│   │   └── any.rs        # Any
│   └── std/
│       └── boxed.rs      # Box<T>
└── macros/
    ├── src/
    │   ├── lib.rs         # 宏入口（export_class / export_lib / export_pod）
    │   ├── export_class.rs
    │   ├── export_lib.rs
    │   └── export_pod.rs
    └── docs/
        └── design.md      # 详细设计文档
```

---

## v6 新增能力速查（Phase A–C）

> 以下内容对应 cpp2rust-demo v6 方案中 Phase A/B/C 落地的生成能力，均通过环境变量开关控制（默认关闭，不影响默认产物）。

### 1. 模板类泛型骨架（`CPP2RUST_GEN_TEMPLATES=1`）

将 C++ 模板类映射为 hicc 泛型 `import_class!` 骨架：

```rust
// cpp2rust-todo[TMPL]: 模板类泛型骨架，请按实际实例化类型校验签名与 AbiType 约束；
// 构造函数/静态方法需在 import_lib! 中声明，复杂依赖类型（如 T::OutputRef）请手动补全。
#[cpp(class = "template<class T> Stack<T>")]
pub class Stack<T> {
    #[cpp(method = "void push(T value)")]
    fn push(&mut self, value: T);

    #[cpp(method = "T top() const")]
    fn top(&self) -> T;

    #[cpp(method = "bool empty() const")]
    fn empty(&self) -> bool;
}
```

同时为检测到的模板实例化（字段/方法参数/显式实例化/局部变量）生成类型别名与构造工厂：

```rust
// cpp2rust-todo[TMPL]: 以下为模板实例化别名骨架，请确认实参类型与 AbiType 约束；
pub type StackI32 = Stack<hicc::Pod<i32>>;

// cpp2rust-todo[TMPL]: StackI32 构造工厂骨架
#[cpp(func = "Stack<int>* stack_i32_new(int initial)")]
pub unsafe fn stack_i32_new(initial: i32) -> StackI32;
```

### 2. 模板函数泛型骨架（`CPP2RUST_GEN_TEMPLATES=1`）

将 C++ 模板函数映射为 `import_lib!` 中的泛型占位骨架：

```rust
// cpp2rust-todo[TMPL]: 模板函数需按实例化类型声明（如 do_swap<int>(int*, int*)）；
// 下方 <T> 为泛型占位，请替换为实际实例化类型并确认安全性。
#[cpp(func = "void do_swap<T>(T*, T*)")]
pub unsafe fn do_swap(a: *mut T, b: *mut T);
```

### 3. `@make_proxy` 代理工厂（`CPP2RUST_GEN_PROXY=1`）

为继承纯虚接口的具体类生成 Rust 侧实现 C++ 抽象类的代理工厂骨架：

```rust
// cpp2rust-todo[PROXY]: @make_proxy 代理工厂骨架 —— 让 Rust 实现 C++ 抽象类 IFoo；
// 需先用 #[interface] 定义对应 Trait，再在 @make_proxy 工厂中创建 Rust 实现。
#[cpp(func = "IFoo* @make_proxy<IFoo, ConcreteFoo>()")]
#[interface(name = "IFoo")]
pub unsafe fn make_concrete_foo() -> ConcreteFoo;
```

### 4. `@dynamic_cast` 下行转换（`CPP2RUST_GEN_DYNAMIC_CAST=1`）

为「继承多态基类的派生类」生成裸指针形式与引用形式下行转换骨架，覆盖直接基类与跨层间接祖先：

```rust
// cpp2rust-todo[DCAST]: @dynamic_cast 下行转换骨架 —— 多态基类 Base 向下转换为派生类 Derived；
// 转换失败返回空指针，调用方需判空（is_null）。RTTI 要求源类型为多态类型（含虚函数）。
#[cpp(func = "const Derived* @dynamic_cast<const Derived*>(const Base*)")]
pub unsafe fn dynamic_cast_base_to_derived(src: *const Base) -> *const Derived;
// cpp2rust-todo[DCAST]: 引用形式 —— 仅在转换必定成功时使用；否则请用上面的裸指针形式判空。
#[cpp(func = "const Derived* @dynamic_cast<const Derived*>(const Base*)")]
pub unsafe fn dynamic_cast_base_to_derived_ref(src: &Base) -> &Derived;
```

**注意**：引用形式要求转换必定成功；若转换可能失败，必须使用裸指针形式并判空（`is_null()`）。

### 5. 冒烟测试生成（`CPP2RUST_GEN_SMOKE`，默认开启）

`init` 阶段在 `.cpp2rust/<feature>/rust/tests/smoke.rs` 自动生成冒烟测试：
- 对所有 `pub class` 类型生成编译期类型可用性断言；
- 对无法从集成测试直接调用的工厂/全局函数生成 `cpp2rust-todo[SMOKE]` 占位说明；
- 设置 `CPP2RUST_GEN_SMOKE=0` 可关闭生成。

### 6. 环境变量汇总

| 环境变量 | 默认值 | 说明 |
|---------|--------|------|
| `CPP2RUST_GEN_TEMPLATES` | 关闭 | 生成模板类/函数泛型骨架与实例化别名 |
| `CPP2RUST_GEN_PROXY` | 关闭 | 生成 `@make_proxy` 代理工厂骨架 |
| `CPP2RUST_GEN_DYNAMIC_CAST` | 关闭 | 生成 `@dynamic_cast` 下行转换骨架（含引用形式） |
| `CPP2RUST_GEN_SMOKE` | 开启 | 生成 `tests/smoke.rs` 冒烟测试（`0`/`false`/`no`/`off` 关闭） |
