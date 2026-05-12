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

## 针对模板密集型库的配置建议

当目标 C++ 库大量使用模板（如 RapidJSON、Abseil、fmt）时，需要额外配置才能充分利用 cpp2rust-demo 的提取能力。

### alias-unlocking 模式

**核心思路**：让 clang 在处理 entry.cpp 时能看到 `typedef`/`using` 别名声明，cpp2rust-demo 会将这些别名注册到 AliasRegistry，进而解锁模板特化的提取和方法参数类型门。

```cpp
// entry.cpp — header-only 库的 synthetic translation unit
// 目的：让 clang 看到所有需要绑定的 typedef 别名。

// 1. 包含目标头文件（库自带的 typedef 会自动注册）。
#include "rapidjson/document.h"   // 包含 Document / Value 等别名
#include "rapidjson/writer.h"     // 包含 Writer<StringBuffer> 等
#include "rapidjson/prettywriter.h"
#include "rapidjson/stringbuffer.h"
#include "rapidjson/error/error.h"

// 2. 如果需要自定义 allocator 特化，手动添加别名。
// using FastDoc = rapidjson::GenericDocument<
//     rapidjson::UTF8<char>,
//     rapidjson::MemoryPoolAllocator<rapidjson::CrtAllocator>>;
```

**init 命令**（以 RapidJSON 为例）：
```bash
cpp2rust-demo init --link rapidjson --no-link \
    --extra-clang-args "-std=c++11 -I/path/to/rapidjson/include" \
    -- clang++ -x c++ -std=c++11 -fsyntax-only \
         -I/path/to/rapidjson/include entry.cpp
```

### RapidJSON 专用 entry.cpp 模板

以下 entry.cpp 覆盖 RapidJSON 全部主要绑定场景（`Document` / `Value` / `Writer` / `PrettyWriter` / `StringBuffer` / `Pointer` / `Schema` / 枚举）：

```cpp
// rapidjson-entry.cpp
// 一站式触发 RapidJSON 的全部主要 typedef 别名。
#include "rapidjson/document.h"       // Document, Value, Member, Type
#include "rapidjson/writer.h"         // Writer<StringBuffer>
#include "rapidjson/prettywriter.h"   // PrettyWriter<StringBuffer>
#include "rapidjson/stringbuffer.h"   // StringBuffer
#include "rapidjson/pointer.h"        // Pointer, GenericPointer
#include "rapidjson/schema.h"         // SchemaDocument, SchemaValidator
#include "rapidjson/error/error.h"    // ParseErrorCode (enum)
#include "rapidjson/error/en.h"       // GetParseError_En
```

运行后，AliasRegistry 会自动注册以下映射（从 RapidJSON 头文件中的 `typedef` 收集）：

| 模板名 | 别名 |
|--------|------|
| `GenericDocument` | `Document` |
| `GenericValue` | `Value` |
| `GenericMember` | `Member` |
| `GenericWriter` | `Writer` |
| `GenericStringBuffer` | `StringBuffer` |
| `GenericPointer` | `Pointer` |

### 哪些操作必须通过 C++ shim

以下 C++ 特性 hicc 本身不支持，必须手写 C++ 包装函数：

| 特性 | 原因 | 建议方案 |
|------|------|---------|
| `operator[]`、`operator=` 等 | hicc 不识别 C++ 运算符名 | 自动生成的 `operator_shims.hpp` 提供 starter；补全后通过 `#[cpp(func = "...")]` 绑定 |
| 自定义 allocator 注入 | 需要运行时传参，难以通过静态类型映射 | 写 C++ 工厂函数封装构造，暴露为 `import_lib!` 自由函数 |
| `std::string` 返回 | 跨 ABI 不安全（SSO 布局不稳定） | 写 C++ shim 将结果复制到 `const char*` 或通过输出参数传出 |
| `std::function`/lambda 参数 | hicc 无法表达 | 在 C++ 侧封装为虚函数接口，再用 `@make_proxy` 反向绑定 |
| 变参函数（`printf`-style） | Rust FFI 不支持 C 可变参数 | 写固定参数的 C++ 包装，或使用 `libc::printf` |

### operator shim 三步工作流

1. **工具自动生成 starter**（`meta/<feature>/operator_shims.hpp` 和 `free/shim_ops.rs`）：
   ```cpp
   // operator_shims.hpp（自动生成）
   static inline JsonValue& json_value_assign(JsonValue& self, const JsonValue& rhs) {
       return self = rhs;
   }
   ```

2. **用户确认/调整实现**：检查 `operator_shims.hpp`，对复杂场景补充逻辑。

3. **在 `hicc::cpp!` 中引入并使用**：
   ```rust
   // build.rs 中添加 shim header 的 include 路径
   hicc_build::Build::new()
       .include(".cpp2rust/default/meta")  // operator_shims.hpp 所在目录
       .compile(...);

   // src/... 中引用 shim_ops.rs 的绑定
   use crate::mod_entry::free::shim_ops::*;
   ```

