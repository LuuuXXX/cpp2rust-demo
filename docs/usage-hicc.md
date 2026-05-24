# Hicc 使用指南

## 简介

`hicc` 是一个用于 Rust 和 C++ 混合编程的项目，旨在将 C++ 接口转换为 Rust API。通过 hicc，开发者可以在 Rust 代码中安全地调用 C++ 代码，实现两种语言之间的无缝互操作。

hicc 包含三个核心组件：

| 组件 | 说明 |
|------|------|
| **`hicc`** | C++ 接口转换为 Rust API 的基础功能，提供核心的 `cpp!`、`import_lib!`、`import_class!` 宏 |
| **`hicc-build`** | 基于 hicc 的构建工具，包含 C++ 代码编译和链接功能 |
| **`hicc-std`** | C++ 标准库容器（vector、map、set 等）的 Rust 封装 |

### 为什么需要 hicc？

直接使用 Rust FFI 调用 C++ 代码存在以下问题：

1. **内存布局未知**：Rust 不知道 C++ 类的内存结构
2. **所有权问题**：C++ 有构造函数、析构函数、移动语义，Rust 没有
3. **异常处理**：C++ 异常机制与 Rust 的 `Result` 机制不兼容
4. **模板代码**：C++ 模板实例化在编译时完成，Rust 泛型在运行时解析

hicc 通过指针和接口抽象解决了这些问题，让你可以在 Rust 中安全地调用 C++ 代码。

## 核心概念

### 1. C++ 类在 Rust 中的表示

由于 Rust 无法感知 C++ 类的内存布局，hicc 使用**指针**来访问 C++ 对象：

```rust
// hicc 内部表示
struct CppObject {
    methods: &'static CppObjectMethods,  // 方法表
    obj: *const (),                      // C++ 对象的指针
}
```

这意味着：
- 所有 C++ 对象在 Rust 侧都表示为**指针**
- 对象的方法调用通过方法表进行动态分派
- Rust 负责管理指针（引用计数），C++ 侧负责实际对象生命周期

### 2. 指针和引用类型映射

| C++ 返回类型 | Rust 返回类型 | 说明 |
|------------|--------------|------|
| `T*` | `ClassMutPtr<'static, T>` | 可变指针，可修改对象 |
| `const T*` | `ClassRef<'static, T>` | 只读指针，不可修改 |
| `T&&` | `T` | 所有权转移（移动语义） |
| `T&` | `&mut T` / `&T` | 引用，自动适配 |

### 3. ABI 包装器：`AbiClass<T>`

对于需要在 Rust 侧直接管理的 C++ 对象，使用 `AbiClass<T>`：

```rust
// 在 Rust 内存中创建 C++ 对象
let cpp_string = AbiClass::<std::string>::from_raw_parts(ptr, len);
```

## 快速开始

### 环境准备

```bash
# 克隆 hicc 项目
git clone https://github.com/DBJ2rics/hicc.git
cd hicc

# 构建 hicc
cargo build --release
```

### 步骤一：创建 Rust 项目

```bash
cargo new my-ffi-project
cd my-ffi-project
```

### 步骤二：添加依赖

```toml
# Cargo.toml
[dependencies]
hicc = { path = "../hicc/hicc" }

[build-dependencies]
hicc-build = { path = "../hicc/hicc-build" }
```

### 步骤三：编写 C++ 接口（main.rs）

```rust
// src/main.rs

// 1. 包含 C++ 头文件
hicc::cpp! {
    #include <iostream>
    #include <string>

    // 可以直接定义 C++ 函数
    static void hello() {
        std::cout << "Hello from C++!" << std::endl;
    }
}

// 2. 声明要导入的函数和库
hicc::import_lib! {
    #![link_name = "my_lib"]  // 链接的 C++ 库名

    // 声明 C++ 类（forward declaration）
    class MyClass;

    // 声明全局函数
    #[cpp(func = "void hello()")]
    fn hello();

    // 声明有返回值的函数
    #[cpp(func = "int add(int, int)")]
    fn add(a: i32, b: i32) -> i32;
}

fn main() {
    // 3. 调用 C++ 函数
    hello();
    let result = add(1, 2);
    println!("1 + 2 = {}", result);
}
```

### 步骤四：编写构建脚本

```rust
// build.rs
fn main() {
    hicc_build::Build::new()
        .rust_file("src/main.rs")  // 指定包含 hicc 宏的文件
        .compile("my_lib");        // C++ 库名称
    println!("cargo::rustc-link-lib=my_lib");
    println!("cargo::rustc-link-lib=stdc++");
}
```

### 步骤五：编写 C++ 实现

```cpp
// cpp/my_lib.cpp
#include <iostream>

class MyClass {
public:
    void greet() { std::cout << "Hello!" << std::endl; }
};

extern "C" {
    void hello() {
        std::cout << "Hello from C++!" << std::endl;
    }

    int add(int a, int b) {
        return a + b;
    }
}
```

### 步骤六：编译和运行

```bash
# 编译 C++ 共享库
g++ -shared -fPIC -o libmy_lib.so cpp/my_lib.cpp

# 编译 Rust 项目
RUSTFLAGS="-L ." cargo run
```

## 宏接口详解

hicc 提供了三个核心宏来实现 Rust-C++ 互操作。

### `hicc::cpp!` - 内联 C++ 代码

用于在 Rust 源代码中直接嵌入 C++ 代码。

**常用场景**：

1. **包含头文件**
   ```rust
   hicc::cpp! {
       #include <iostream>
       #include <vector>
       #include <string>
   }
   ```

2. **定义内联 C++ 函数**
   ```rust
   hicc::cpp! {
       #include <iostream>

       static void log_message(const char* msg) {
           std::cout << "[LOG] " << msg << std::endl;
       }

       static int add(int a, int b) {
           return a + b;
       }
   }
   ```

3. **定义完整的 C++ 类**
   ```rust
   hicc::cpp! {
       #include <string>

       class MyClass {
       private:
           std::string name;
       public:
           MyClass(const char* n) : name(n) {}
           const char* get_name() const { return name.c_str(); }
       };
   }
   ```

### `hicc::import_lib!` - 声明外部库函数和符号

**核心作用**：从 C++ 库导入全局函数、全局变量，以及类的**前向声明**。

**与 `import_class!` 的区别**：

| 特性 | `import_lib!` | `import_class!` |
|------|---------------|------------------|
| 声明内容 | 全局函数、全局变量、类前向声明 | 完整的类定义（生成 Rust struct） |
| 生成代码 | 函数调用包装器 | Rust struct、impl 块、Drop/Debug 实现 |
| 配合使用 | 提供 forward declaration | 提供完整类型定义 |

**基本语法**：
```rust
hicc::import_lib! {
    #![link_name = "库名称"]  // 链接的 C++ 库名

    // 类前向声明（告诉 Rust 这个类存在）
    class MyClass;

    // 全局函数声明
    #[cpp(func = "C++函数签名")]
    fn rust函数名(参数) -> 返回类型;

    // 全局变量声明
    #[cpp(data = "global_counter")]
    fn get_global_counter() -> &'static i32;
}
```

**示例**：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    // 声明类
    class Rectangle;

    // 声明全局函数
    #[cpp(func = "void init()")]
    fn init();

    // 声明带参数的函数
    #[cpp(func = "int max(int, int)")]
    fn max(a: i32, b: i32) -> i32;

    // 声明返回指针的函数
    #[cpp(func = "const char* get_name()")]
    fn get_name() -> *const u8;

    // 声明模板函数（完整签名）
    #[cpp(func = "std::unique_ptr<std::string> std::make_unique<std::string, const char*>(const char*&&)")]
    unsafe fn make_string(s: *const u8) -> String;
}
```

### `hicc::import_class!` - 定义 C++ 类的 Rust 表示

**核心作用**：为 C++ 类生成完整的 Rust 类型定义，包括：
- Rust struct（对应 C++ 类）
- 方法实现（通过方法表调用 C++ 方法）
- `Drop` trait（自动释放 C++ 对象）
- `Debug` trait（调试输出）
- `AbiClass` trait（ABI 接口）

**基本语法**：
```rust
hicc::import_class! {
    #[cpp(class = "C++类名")]
    class RustName {
        #[cpp(method = "C++方法签名")]
        fn 方法名(&self, 参数) -> 返回类型;
    }
}
```

**重要**：`import_class!` 生成的是类型定义，不是函数声明。类的方法调用实际通过 FFI 进行。

**示例**：

```rust
hicc::import_class! {
    // 普通类
    #[cpp(class = "Rectangle")]
    class Rectangle {
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "void resize(double, double)")]
        fn resize(&mut self, w: f64, h: f64);
    }

    // 带构造函数的类
    #[cpp(class = "Point", ctor = "Point(double, double)")]
    class Point {
        #[cpp(method = "double x() const")]
        fn x(&self) -> f64;

        #[cpp(method = "double y() const")]
        fn y(&self) -> f64;
    }

    // 抽象类（接口）
    #[interface]
    class Shape {
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "void draw() const")]
        fn draw(&self);
    }
}
```

### `#[method]` 属性

将全局函数包装为类的成员方法：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    // 工厂函数包装为构造函数
    #[cpp(func = "std::unique_ptr<MyClass> create_my_class(int)")]
    #[method(class = MyClass, name = new)]
    unsafe fn create_my_class(val: i32) -> MyClass;
}
```

## 高级用法

### 1. 继承 C++ 抽象类（接口）

当 C++ 有抽象类（纯虚函数）时，使用 `#[interface]` 声明接口：

```rust
// C++ 端
hicc::cpp! {
    class Drawable {
    public:
        virtual ~Drawable() {}
        virtual void draw() const = 0;
        virtual double area() const = 0;
    };
}

hicc::import_class! {
    #[interface]
    class Drawable {
        #[cpp(method = "void draw() const")]
        fn draw(&self);

        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;
    }
}
```

在 Rust 中实现 C++ 接口（使用 `@make_proxy`）：

```rust
// Rust 实现了 C++ 接口
struct RustDrawable;

impl Drawable for RustDrawable {
    fn draw(&self) {
        println!("Drawing from Rust!");
    }
    fn area(&self) -> f64 {
        42.0
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "Drawable @make_proxy<Drawable>()")]
    #[interface(name = "Drawable")]
    fn create_rust_drawable(intf: hicc::Interface<Drawable>) -> Drawable;
}

fn main() {
    let drawable = create_rust_drawable(RustDrawable);
    drawable.draw();
}
```

### 2. 捕获 C++ 异常

C++ 异常会被捕获并转换为 `hicc::Exception<T>`：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "int divide(int, int)")]
    fn divide(a: i32, b: i32) -> hicc::Exception<i32>;
}

fn main() {
    match divide(10, 2) {
        Ok(result) => println!("10 / 2 = {}", result),
        Err(e) => println!("Exception occurred: {:?}", e),
    }

    // 捕获除零异常
    match divide(10, 0) {
        Ok(_) => println!("Success"),
        Err(_) => println!("Caught division by zero!"),
    }
}
```

### 3. 传递 Rust 闭包到 C++（std::function）

```rust
hicc::import_lib! {
    #![link_name = "example"]

    // C++ 接受 std::function
    #[cpp(func = "int transform(int, std::function<int(int)>)")]
    fn transform(
        value: i32,
        func: hicc::Function<fn(i32) -> i32>
    ) -> i32;
}

fn main() {
    // 传递 Rust 闭包
    let result = transform(10, |x: i32| -> i32 {
        x * 2
    }.into());
    println!("transform(10, x*2) = {}", result);

    // 链式调用
    let result = transform(5, |x| x + 1);
    let result = transform(result, |x| x * 3);
}
```

### 4. `dynamic_cast` 类型转换

```rust
hicc::import_class! {
    #[cpp(class = "Base", ctor = "Base()")]
    class Base {
        #[cpp(method = "const char* get_type() const")]
        fn get_type(&self) -> *const u8;
    }

    #[cpp(class = "Derived", ctor = "Derived()")]
    class Derived: Base {
        #[cpp(method = "int get_value() const")]
        fn get_value(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    // dynamic_cast 转换
    #[cpp(func = "Derived* @dynamic_cast<Derived*>(Base*)")]
    fn base_to_derived(base: *mut Base) -> *mut Derived;
}
```

### 5. Placement New（在 Rust 内存中创建 C++ 对象）

当 C++ 对象需要在 Rust 分配的内存中创建时使用：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "hicc::AbiClass<std::string> hicc::placement_new<std::string, const char*>(void*, size_t, const char*&&)")]
    fn construct_string(buf: *mut u8, len: usize, data: *const u8) -> &'static mut String;
}

fn main() {
    // 在 Rust 栈内存中构建 C++ 对象
    let mut buffer = [0u8; 256];
    let cpp_string = construct_string(
        buffer.as_mut_ptr(),
        buffer.len(),
        b"Hello World\0".as_ptr()
    );
    println!("String length: {}", cpp_string.len());
}
```

### 6. 读写 C++ 全局变量

```rust
hicc::import_lib! {
    #![link_name = "example"]

    // 读取全局变量
    #[cpp(data = "global_counter")]
    fn get_global_counter() -> &'static i32;

    // 写入全局变量
    #[cpp(data = "global_counter")]
    fn set_global_counter(val: &mut i32);
}

fn main() {
    let counter = get_global_counter();
    println!("Global counter = {}", counter);

    let mut counter = get_global_counter();
    *counter = 42;
}
```

### 7. 私有析构函数的类

有些 C++ 类使用自定义的内存管理（如对象池），需要特殊处理：

```rust
hicc::import_class! {
    #[cpp(class = "PooledObject", destroy = "PooledObject::release")]
    class PooledObject {
        fn acquire() -> Self { unsafe { acquire_pooled_object() } }
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "PooledObject* acquire_pooled_object()")]
    unsafe fn acquire_pooled_object() -> PooledObject;
}
```

### 8. 忽略缺省参数和返回值

```rust
// C++ 函数有缺省参数
hicc::cpp! {
    int configure(int port, int timeout = 30) { return port + timeout; }
}

hicc::import_lib! {
    #![link_name = "example"]

    // 只传递必需参数
    #[cpp(func = "int configure(int, int)")]
    fn configure(port: i32, timeout: i32) -> i32;
}

fn main() {
    // 调用时只需传一个参数
    let result = configure(8080, 60);
}
```

## hicc-std：标准库容器封装

hicc-std 提供了 C++ 标准库容器的 Rust 封装，让你可以用 Rust 的方式操作 C++ 容器。

### 支持的容器类型

| 容器 | 说明 |
|------|------|
| `std::vector<T>` | 动态数组 |
| `std::list<T>` | 双向链表 |
| `std::map<K, V>` | 有序映射 |
| `std::set<T>` | 有序集合 |
| `std::unordered_map<K, V>` | 哈希映射 |
| `std::unordered_set<T>` | 哈希集合 |
| `std::deque<T>` | 双端队列 |
| `std::string` | 字符串 |
| `std::stack<T>` | 栈 |
| `std::queue<T>` | 队列 |

### 使用示例

```rust
hicc::cpp! {
    #include <hicc/std/map.hpp>
    #include <hicc/std/string.hpp>
    typedef std::map<int, std::string> IntStringMap;
}

hicc::import_lib! {
    #![link_name = "example"]

    class IntStringMap = hicc_std::map<hicc::Pod<i32>, hicc_std::string>;

    #[cpp(func = "std::unique_ptr<IntStringMap> hicc::make_unique<IntStringMap>()")]
    fn create_map() -> IntStringMap;
}

fn main() {
    let mut map = create_map();

    // 插入数据
    let key = hicc::Pod::new(1);
    let value = hicc_std::string::from(b"one\0");
    map.insert(&key, &value);

    // 查找
    let result = map.get(&key);
    match result {
        Some(val) => println!("Found: {:?}", val),
        None => println!("Not found"),
    }
}
```

## 类型映射表

### 基本类型

| C++ 类型 | Rust 类型 |
|---------|----------|
| `int` | `i32` |
| `unsigned int` | `u32` |
| `long` | `i64` / `isize` |
| `unsigned long` | `u64` / `usize` |
| `float` | `f32` |
| `double` | `f64` |
| `char` | `i8` |
| `bool` | `bool` |
| `void` | `()` |

### 指针和引用

| C++ 类型 | Rust 类型 |
|---------|----------|
| `T*` | `*mut T` → `ClassMutPtr` |
| `const T*` | `*const T` → `ClassRef` |
| `T&` | `&mut T` / `&T` |
| `const T&` | `&T` |

### 智能指针

| C++ 类型 | Rust 类型 |
|---------|----------|
| `std::unique_ptr<T>` | `hicc::unique_ptr<T>` |
| `std::shared_ptr<T>` | `hicc::shared_ptr<T>` |
| `std::weak_ptr<T>` | `hicc::weak_ptr<T>` |

### STL 容器

| C++ 类型 | Rust 类型 |
|---------|----------|
| `std::vector<T>` | `hicc_std::vector<T>` |
| `std::map<K, V>` | `hicc_std::map<K, V>` |
| `std::set<T>` | `hicc_std::set<T>` |
| `std::string` | `hicc_std::string` |

## 完整示例

### 示例：包装一个 C++ Rectangle 类

**1. C++ 头文件（rectangle.hpp）**

```cpp
#pragma once

class Rectangle {
private:
    double width;
    double height;

public:
    Rectangle();
    Rectangle(double w, double h);

    double area() const;
    double perimeter() const;
    void resize(double w, double h);

    double get_width() const { return width; }
    double get_height() const { return height; }
};
```

**2. C++ 实现（rectangle.cpp）**

```cpp
#include "rectangle.hpp"

Rectangle::Rectangle() : width(0), height(0) {}
Rectangle::Rectangle(double w, double h) : width(w), height(h) {}

double Rectangle::area() const {
    return width * height;
}

double Rectangle::perimeter() const {
    return 2 * (width + height);
}

void Rectangle::resize(double w, double h) {
    width = w;
    height = h;
}
```

**3. Rust FFI（src/main.rs）**

```rust
hicc::cpp! {
    #include "rectangle.hpp"
}

// 用 import_class! 定义 Rectangle 类的结构和所有方法
hicc::import_class! {
    #[cpp(class = "Rectangle")]
    class Rectangle {
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "double perimeter() const")]
        fn perimeter(&self) -> f64;

        #[cpp(method = "void resize(double, double)")]
        fn resize(&mut self, w: f64, h: f64);

        #[cpp(method = "double get_width() const")]
        fn get_width(&self) -> f64;

        #[cpp(method = "double get_height() const")]
        fn get_height(&self) -> f64;
    }
}

// 用 import_lib! 声明工厂函数（创建对象）
hicc::import_lib! {
    #![link_name = "rectangle"]

    class Rectangle;

    // 默认构造函数
    #[cpp(func = "std::unique_ptr<Rectangle> hicc::make_unique<Rectangle>()")]
    fn rectangle_default() -> Rectangle;

    // 带参数的构造函数
    #[cpp(func = "std::unique_ptr<Rectangle> hicc::make_unique<Rectangle, double, double>(double&&, double&&)")]
    unsafe fn rectangle_new(w: f64, h: f64) -> Rectangle;
}

fn main() {
    // 创建默认矩形
    let rect = rectangle_default();
    println!("Default rectangle: {}x{}", rect.get_width(), rect.get_height());

    // 创建指定大小的矩形
    let rect2 = unsafe { rectangle_new(10.0, 5.0) };
    println!("New rectangle: {}x{}", rect2.get_width(), rect2.get_height());

    rect2.resize(8.0, 4.0);
    println!("After resize: {}x{}", rect2.get_width(), rect2.get_height());
    println!("Area: {}", rect2.area());
    println!("Perimeter: {}", rect2.perimeter());
}
```

## 注意事项

1. **C++ 函数声明格式**：参数列表**只包含类型**，不能包含参数名
   ```rust
   // 正确
   #[cpp(func = "int add(int, int)")]
   fn add(a: i32, b: i32) -> i32;

   // 错误
   #[cpp(func = "int add(int a, int b)")]  // 不能有参数名
   ```

2. **模板函数签名**：必须包含完整的模板参数列表
   ```rust
   #[cpp(func = "std::unique_ptr<std::string> std::make_unique<std::string, const char*>(const char*&&)")]
   ```

3. **构建要求**：
   - C++ 编译器需要支持 C++11 或更高版本
   - 必须链接 C++ 标准库（`stdc++`）

4. **生命周期管理**：
   - Rust 通过 `unique_ptr` / `shared_ptr` 自动管理 C++ 对象生命周期
   - 避免在 C++ 对象销毁后继续访问

5. **异常安全**：
   - 使用 `hicc::Exception<T>` 捕获 C++ 异常
   - 未捕获的异常会导致程序终止

## 项目结构

```
hicc/
├── hicc/                  # 核心库
│   ├── src/
│   │   ├── lib.rs        # 主库入口
│   │   ├── class.rs      # 类定义和 ABI 包装
│   │   ├── function.rs   # 函数 FFI 支持
│   │   ├── exception.rs  # 异常处理
│   │   ├── memory.rs     # 内存管理
│   │   ├── generic.rs    # 泛型支持
│   │   └── any.rs        # 类型擦除支持
│   └── include/          # C++ 头文件
├── hicc-autogen/         # 自动代码生成
│   └── src/
│       ├── import_lib.rs     # import_lib! 宏实现
│       ├── import_class.rs   # import_class! 宏实现
│       ├── cpp.rs            # cpp! 宏实现
│       ├── class.rs          # 类结构分析
│       ├── function.rs      # 函数签名解析
│       ├── attr.rs           # 属性解析
│       ├── visitor.rs        # AST 遍历
│       ├── class_visitor.rs # 类访问器
│       └── export/          # 代码导出
├── hicc-build/           # 构建工具
│   └── src/
│       └── lib.rs        # 构建逻辑
├── hicc-macros/          # 过程宏定义
│   └── src/
│       └── lib.rs
├── hicc-rs/              # Rust ABI 工具库
│   └── src/
│       └── lib.rs
├── hicc-std/             # 标准库封装
│   └── src/
│       ├── vector.rs
│       ├── map.rs
│       ├── string.rs
│       └── ...
└── examples/             # 示例项目
    ├── hello-world/
    ├── class/
    └── ...
```

## 常见问题

### Q: 如何调试 FFI 调用？

```rust
// 启用 debug 日志
std::env::set_var("RUST_LOG", "debug");
```

### Q: 如何处理 C++ 端的内存泄漏？

使用 `hicc::unique_ptr` 确保对象在 Rust 释放时自动销毁。

### Q: 能否在 Rust 中继承 C++ 类？

不能直接在 Rust 继承 C++ 类，但可以通过 `#[interface]` 让 Rust 实现 C++ 接口。

### Q: 如何处理 C++ 模板类？

hicc 通过显式实例化支持模板：
```rust
// 显式实例化 int 类型的 Stack
#[cpp(func = "Stack<int>* create_stack_int()")]
fn create_stack_int() -> *mut StackInt;
```
