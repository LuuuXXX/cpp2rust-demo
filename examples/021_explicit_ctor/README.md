# 021_explicit_ctor - 显式构造函数（hicc 直出，无 shim）

## C++ 特性

本示例展示**显式构造函数**的地道 C++ 命名空间类 `Widget`：含两个公有构造
`Widget(int)`（非 `explicit`，允许隐式转换）与 `explicit Widget(double)`（禁止隐式转换）。
用 hicc 直出绑定，**无 opaque 指针、无 `extern "C"` 桥接、无 `*_new`/`*_delete` shim**。

## C++ 代码（节选）

```cpp
namespace explicit_ctor_ns {

class Widget {
public:
    Widget(int v);             // 非 explicit：int 可隐式转换为 Widget
    explicit Widget(double v); // explicit：double 必须显式构造
    ~Widget();
    int getValue() const;
private:
    int value_;
};

} // namespace explicit_ctor_ns
```

配套 `standalone.sh` / `Makefile` 两种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出 + 多构造工厂）

hicc 直出为**每个公有构造**各派生一条 `hicc::make_unique` 工厂与关联函数：第一个构造
映射为 `new`，后续构造映射为 `new_2`、`new_3` …… 本例即 `new(i32)` 与 `new_2(f64)`：

```rust
hicc::import_class! {
    #[cpp(class = "explicit_ctor_ns::Widget")]
    pub class Widget {
        #[cpp(method = "int getValue() const")]
        pub fn get_value(&self) -> i32;

        pub fn new(v: i32) -> Self { widget_new(v) }
        pub fn new_2(v: f64) -> Self { widget_new_2(v) }
    }
}

hicc::import_lib! {
    #![link_name = "explicit_ctor"]
    #[cpp(func = "std::unique_ptr<explicit_ctor_ns::Widget> hicc::make_unique<explicit_ctor_ns::Widget, int>(int&&)")]
    pub fn widget_new(v: i32) -> Widget;
    #[cpp(func = "std::unique_ptr<explicit_ctor_ns::Widget> hicc::make_unique<explicit_ctor_ns::Widget, double>(double&&)")]
    pub fn widget_new_2(v: f64) -> Widget;
}
```

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| 多个公有构造 | 逐个派生 `make_unique` 工厂：`new`、`new_2` …… |
| `Widget(int)` | `new(i32)` |
| `explicit Widget(double)` | `new_2(f64)` |
| `explicit` 关键字 | 只约束 C++ 隐式转换；Rust 侧两者都是显式关联函数调用，不影响绑定 |
| 析构 | 交给 hicc 的 `Drop`，无 `*_delete` shim |

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 总结

1. **多构造工厂**：每个公有构造各对应一条 `make_unique` 工厂与关联函数。
2. **命名规则**：首个构造为 `new`，其余为 `new_2`/`new_3`……
3. **explicit 无关**：`explicit` 只影响 C++ 隐式转换，不改变直出绑定。
4. **去 shim**：无 `*_new`/`*_delete`、无 opaque 指针、无 `extern "C"` 桥接。
