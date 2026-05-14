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
4. 语义层职责：`method` 承接 `import_class!` 实例方法绑定；`free` 承接自由函数/静态方法/make_proxy/全局变量；`types` 负责类型语义（含 C++→Rust 映射与查询函数）；`class` 负责类级语义结构（含关系访问函数）；`common/*` 进入全局 merged 共享语义层；`global` 当前不生成
5. `init --no-link`（`--header-only`）用于 header-only/no-link 场景：生成的 `build.rs` 不会输出 `cargo::rustc-link-lib=<link_name>`
6. destructor、operator overload、template declarations 当前会被跳过，并在 `init-interface-report.md` 中显示 skipped 原因；virtual/pure virtual 方法按以下规则处理：非纯 virtual 直接提取、全纯虚类生成 `#[interface]` trait、混合类的纯虚方法提取为 companion `#[interface]` 并加入继承链
7. `inline` 函数对工具透明，与普通函数生成完全相同的绑定（见 [`examples/features/01-inline-functions/`](../examples/features/01-inline-functions/README.md)）
8. 方法 `self` 类型映射规则：`const` 方法 → `&self`；普通方法 → `&mut self`；`&&` 右值引用方法 → `self`（消耗对象，见 [`examples/features/03-rvalue-ref/`](../examples/features/03-rvalue-ref/README.md)）

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

### `va_list` 变参函数（P3）

最后参数为 `va_list` 的函数会被自动提取为 `unsafe fn`，`va_list` 参数被丢弃并以 `...` 替代：

```rust
// C++: void log_message(int level, va_list args);
#[cpp(func = "void log_message(int)")]
unsafe fn log_message(level: i32, ...);
```

> 注意：纯 `...` 变参（如 `printf(const char*, ...)`）暂不支持，需手写固定参数 C++ 包装。

### 函数指针参数提示（P3）

含函数指针参数的函数会被跳过，并在接口报告（`meta/init-interface-report.md`）中生成对应的虚函数接口骨架 + `@make_proxy` 使用提示：

```cpp
// 接口报告中自动生成：
// struct CallbackHandler {
//   // Underlying type: void (*)(int)
//   virtual /* return_type */ call(/* args */) = 0;
//   virtual ~CallbackHandler() = default;
// };
// Replace `callback` parameter with `CallbackHandler *` and forward the call.
// Use hicc @make_proxy to implement the interface from Rust.
```

### `@dynamic_cast` 骨架（P3）

当存在类继承关系时，工具自动在 `free/dynamic_casts.rs` 生成注释掉的 `@dynamic_cast` 绑定骨架。解注释所需的行并重新构建即可：

```rust
// free/dynamic_casts.rs（自动生成，解注释你需要的部分）
hicc::import_lib! {
    #![link_name = "mylib"]

    // Cast Shape* → Circle* (returns null if types don't match).
    // #[cpp(func = "Circle @dynamic_cast<Circle>(Shape *)")]
    // fn dynamic_cast_to_circle(ptr: *mut Shape) -> *mut Circle;
}
```

## Cargo 依赖

```toml
[dependencies]
hicc = "0.2.3"

[build-dependencies]
hicc-build = "0.2.1"
```

## 针对模板密集型库的配置建议

当目标 C++ 库大量使用模板（如 RapidJSON、Abseil）时，需确保 clang 在处理 entry.cpp 时能看到 `typedef`/`using` 别名声明，工具才能解锁模板特化提取。核心原则：在 entry.cpp 中 `#include` 包含别名定义的头文件，或手动添加 `using` 别名（如 `using FastDoc = rapidjson::GenericDocument<...>`）。详细机制见 `docs/design.md` 「解锁模板类提取」章节。

**init 命令**（以 RapidJSON 为例）：
```bash
cpp2rust-demo init --link rapidjson --no-link \
    --extra-clang-args "-std=c++11 -I/path/to/rapidjson/include" \
    -- clang++ -x c++ -std=c++11 -fsyntax-only \
         -I/path/to/rapidjson/include entry.cpp
```

### 必须通过 C++ shim 处理的特性

以下特性 hicc 本身不支持，需手写 C++ 包装函数：

| 特性 | 建议方案 |
|------|---------|
| `operator[]`、`operator=` 等 | 工具自动生成 `operator_shims.hpp` starter；补全后通过 `#[cpp(func = "...")]` 绑定 |
| 自定义 allocator 注入 | 写 C++ 工厂函数封装构造，暴露为 `import_lib!` 自由函数 |
| `std::string` 返回 | 写 C++ shim 将结果复制到 `const char*` 或通过输出参数传出 |
| `std::function`/lambda 参数 | 在 C++ 侧封装为虚函数接口，再用 `@make_proxy` 反向绑定 |
| 函数指针参数 | 工具自动在接口报告中生成虚函数接口骨架（`FooHandler { virtual call(...) = 0; }`）+ `@make_proxy` 提示 |
| `va_list` 变参函数 | ✅ P3 已自动支持：最后参数为 `va_list` 时直接生成 `unsafe fn foo(fixed_params, ...) -> T` |
| 纯 `...` 变参函数（`printf`-style，无 `va_list` 参数） | 写固定参数的 C++ 包装，或使用 `libc::printf` |

### operator shim 工作流

1. **工具自动生成 starter**（`.cpp2rust/<feature>/meta/operator_shims.hpp` 和 `free/shim_ops.rs`）
2. **检查/补全实现**：对复杂场景在 `operator_shims.hpp` 中补充逻辑
3. **引入并使用**：
   ```rust
   // build.rs 中加入 shim header 的 include 路径
   hicc_build::Build::new()
       .include(".cpp2rust/default/meta")
       .compile(...);

   // Rust 侧引用 shim_ops.rs 的绑定
   use crate::mod_entry::free::shim_ops::*;
   ```

