# 007_class_constructor - 多构造函数（hicc 直出，无 shim）

## C++ 特性

本示例展示带**多个构造函数 + 析构函数**的地道 C++ 命名空间类，
用 hicc 直出方式绑定：每个公有构造派生一条 `hicc::make_unique` 工厂，
析构交给 hicc 的 `Drop`，**无 opaque 指针、无 `extern "C"` 桥接**。

## C++ 代码（节选）

```cpp
namespace class_ctor_ns {

class Widget {
public:
    Widget();                              // 默认构造
    explicit Widget(int v);                // int 构造
    Widget(std::string n, int v);          // string + int 构造
    ~Widget();                             // 析构

    const std::string& name() const;
    int value() const;
};

} // namespace class_ctor_ns
```

配套 `standalone.sh` / `Makefile` / `CMakeLists.txt` 三种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出）

```rust
hicc::cpp! {
    #include "class_constructor.h"
    #include <hicc/std/string.hpp>
}

hicc::import_class! {
    class string = hicc_std::string;

    #[cpp(class = "class_ctor_ns::Widget")]
    pub class Widget {
        #[cpp(method = "const std::string& name() const")]
        pub fn name(&self) -> &string;

        #[cpp(method = "int value() const")]
        pub fn value(&self) -> i32;

        pub fn new() -> Self { widget_default() }
        pub fn from_int(v: i32) -> Self { widget_from_int(v) }
        pub fn from_named(name: string, v: i32) -> Self { widget_from_named(name, v) }
    }
}

hicc::import_lib! {
    #![link_name = "class_constructor"]

    #[cpp(func = "std::unique_ptr<class_ctor_ns::Widget> hicc::make_unique<class_ctor_ns::Widget>()")]
    pub fn widget_default() -> Widget;

    #[cpp(func = "std::unique_ptr<class_ctor_ns::Widget> hicc::make_unique<class_ctor_ns::Widget, int>(int&&)")]
    pub fn widget_from_int(v: i32) -> Widget;

    #[cpp(func = "std::unique_ptr<class_ctor_ns::Widget> hicc::make_unique<class_ctor_ns::Widget, std::string, int>(std::string&&, int&&)")]
    pub fn widget_from_named(name: hicc_std::string, v: i32) -> Widget;
}
```

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| 多个公有构造 | 每个构造一条 `hicc::make_unique<T, Args...>` 工厂 + `import_class!` body 内关联函数转发 |
| 模板/调用实参 | 模板实参用衰减类型（`int`、`std::string`），调用实参用转发类型（`int&&`、`std::string&&`） |
| 析构函数 | 由 hicc `Drop` 自动负责，无需 `*_delete` |
| `const std::string&` 返回 | `&string`（`ClassRef`，配 `class string = hicc_std::string;`） |

> 工具默认产物（`lib_scaffold.rs`）只默认生成参数可直出映射的构造（`new`/`new_2`）与基本类型方法；
> 含 `std::string` 的构造与返回引用方法由手写 `lib.rs` 用 `hicc_std::string` 补全。

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run / cmake）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 总结

1. **多工厂**：N 个公有构造 → N 条 `make_unique` 工厂，替代多套 `*_new` shim。
2. **Drop 析构**：hicc 自动析构，替代 `*_delete` shim。
3. **转发语义**：模板实参衰减、调用实参 `T&&`，与 hicc 蓝本一致。
4. **零 unsafe 调用**：安全 Rust API。
