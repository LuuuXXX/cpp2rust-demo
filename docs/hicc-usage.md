# hicc 使用说明（cpp2rust-demo）

`cpp2rust-demo` 生成的 Rust 代码统一基于 `hicc`：

- `hicc::cpp!`：引入 `*.cpp2rust` 中间件（例如 `a.cpp.cpp2rust`）
- `hicc::import_class!`：映射 C++ 类实例方法，支持构造函数（`ctor = "..."`）和继承（`class Foo: Bar`）
- `hicc::import_lib!`：映射自由函数、静态方法、全局变量（`#[cpp(data = "...")]`）以及 `@make_proxy` 反向继承绑定
- `hicc_build::Build`：在 `build.rs` 中驱动适配层生成与编译

## 关键约定

1. 每个 `mod_<group>/include/mod.rs` 都会包含 `hicc::cpp! { #include "*.cpp2rust" }`
2. `build.rs` 会为 `hicc_build::Build` 注入中间件所在目录的 include path，并始终引用 `src/...` 活跃视图路径
3. `merge` 后会生成 `rust/src.2/mod_<group>.rs` 与 `rust/src.2/merged_ffi.rs`，并将 `rust/src` 切换到 `src.2`（因此 `build.rs` 无需改成 `src.2/...`）
4. 当前版本中：`method` 是 `import_class!` 的唯一承接层（实例方法绑定），`free` 负责自由函数/静态方法/make_proxy/全局变量；`types` 负责类型语义（含 C++→Rust 映射与查询函数）并进入 merged，`class` 负责类级语义结构（含关系访问函数）并进入 merged，`common/*` 会进入全局 merged 共享语义层（含共享查询函数）；`global` 在本 PR 范围内明确 defer
5. `init --no-link`（`--header-only`）用于 header-only/no-link 场景：生成的 `build.rs` 不会输出 `cargo::rustc-link-lib=<link_name>`
6. destructor、operator overload、template declarations 当前会被跳过，并在 `init-interface-report.md` 中显示 skipped 原因；virtual/pure virtual 方法按以下规则处理：非纯 virtual 直接提取、全纯虚类生成 `#[interface]` trait、混合类的纯虚方法保守跳过

## 新特性：充分利用 hicc 能力

### 构造函数（`ctor = "..."`）

cpp2rust-demo 现在会从 C++ 头文件中提取 public 构造函数，并将最简单的一个（参数最少）作为 hicc 的主要构造函数：

```rust
hicc::import_class! {
    #[cpp(class = "Widget", ctor = "Widget(int)")]
    class Widget {
        #[cpp(method = "void update(double, double)")]
        fn update(&mut self, x: f64, y: f64);
    }
}
```

当有多个可用构造函数时，其余的会在 `import_lib!` 中以工厂函数形式暴露：

```rust
hicc::import_lib! {
    #![link_name = "mylib"]
    class Widget;
    #[member(class = "Widget", method = "new_2")]
    #[cpp(func = "Widget Widget(int, double)")]
    fn widget_new_2(n: i32, scale: f64) -> Widget;
}
```

> 注意：copy 构造函数（`const T &`）和 move 构造函数（`T &&`）会被自动识别并跳过。

### 类继承（`class Foo: Bar`）

当 C++ 类有 public 基类时，cpp2rust-demo 会自动在 `import_class!` 中生成继承语法：

```rust
hicc::import_class! {
    #[interface]
    class Printable {
        #[cpp(method = "void print() const")]
        fn print(&self);
    }

    #[cpp(class = "Document", ctor = "Document()")]
    class Document: Printable {
        #[cpp(method = "void save(const char *)")]
        fn save(&self, path: *const i8);
    }
}
```

### `@make_proxy`（Rust 实现 C++ 抽象类）

对于所有全纯虚抽象类（映射为 `#[interface]` trait），cpp2rust-demo 会自动在 `import_lib!` 中生成对应的 `@make_proxy` 绑定：

```rust
hicc::import_lib! {
    #![link_name = "mylib"]
    // @make_proxy support (for Rust implementations of abstract interfaces).
    #[cpp(func = "Listener @make_proxy<Listener>()")]
    #[interface(name = "Listener")]
    fn new_listener_proxy(intf: hicc::Interface<Listener>) -> Listener;
}
```

使用方式：

```rust
struct MyListener;
impl Listener for MyListener {
    fn on_event(&self) { println!("event!"); }
}
// 将 MyListener 注册为 C++ Listener 子类
let proxy = new_listener_proxy(MyListener);
```

> 注意：使用 `@make_proxy` 时需要在 C++ 侧包含 `<hicc/std/memory.hpp>`，在 `build.rs` 的 include path 中加入 hicc 头文件目录。

### 全局变量（`#[cpp(data = "...")]`）

cpp2rust-demo 现在会从头文件中提取命名空间全局变量，并在 `import_lib!` 中生成 `#[cpp(data = "...")]` 绑定：

```rust
hicc::import_lib! {
    #![link_name = "mylib"]
    // Global variable bindings.
    #[cpp(data = "g_count")]
    fn g_count() -> &'static mut i32;

    #[cpp(data = "myns::VERSION")]
    fn version() -> &'static i32;
}
```

- `const` 变量返回 `&'static T`（只读引用）
- 非 const 变量返回 `&'static mut T`（可写引用）

## Cargo 依赖

```toml
[dependencies]
hicc = "0.2.3"

[build-dependencies]
hicc-build = "0.2.1"
```

