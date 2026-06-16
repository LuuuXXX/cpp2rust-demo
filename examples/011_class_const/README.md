# 011_class_const - const 成员函数（hicc 直出，无 shim）

## C++ 特性

本示例展示带 **const / 非 const 成员函数**的地道 C++ 命名空间类，用 hicc 直出方式绑定：
const 方法保证不修改对象状态、映射为 Rust 的 `&self`，非 const 方法映射为 `&mut self`；
默认构造派生 `hicc::make_unique` 工厂，析构交给 hicc `Drop`，**无 opaque 指针、无 `extern "C"` 桥接**。

## C++ 代码（节选）

```cpp
namespace class_const_ns {

class Calculator {
public:
    Calculator();
    ~Calculator();

    int value() const;          // const：只读 → &self
    int history_count() const;  // const：只读 → &self

    void add(int v);            // 非 const：可变 → &mut self
    void subtract(int v);
    void clear();

private:
    int value_;
    std::vector<int> history_;
};

} // namespace class_const_ns
```

配套 `standalone.sh` / `Makefile` 两种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出）

```rust
hicc::import_class! {
    #[cpp(class = "class_const_ns::Calculator")]
    pub class Calculator {
        #[cpp(method = "int value() const")]
        pub fn value(&self) -> i32;

        #[cpp(method = "int history_count() const")]
        pub fn history_count(&self) -> i32;

        #[cpp(method = "void add(int v)")]
        pub fn add(&mut self, v: i32);

        #[cpp(method = "void subtract(int v)")]
        pub fn subtract(&mut self, v: i32);

        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        pub fn new() -> Self { calculator_new() }
    }
}

hicc::import_lib! {
    #![link_name = "class_const"]

    #[cpp(func = "std::unique_ptr<class_const_ns::Calculator> hicc::make_unique<class_const_ns::Calculator>()")]
    pub fn calculator_new() -> Calculator;
}
```

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| const 成员函数 | `&self`（编译期保证只读借用） |
| 非 const 成员函数 | `&mut self`（可变借用） |
| 默认构造 | `hicc::make_unique<T>` 工厂（`new`） |
| 析构函数 | 由 hicc `Drop` 自动负责 |

> 工具默认产物（`lib_scaffold.rs`）即包含全部 const/非 const 方法与构造工厂，
> 本示例 `lib.rs` 与支架一致，无需手写补全。

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 运行结果

```
Calculator() ctor
value=12 history=3
after clear value=0 history=0
~Calculator() dtor
```

## 总结

1. **const 映射**：const 方法 → `&self`，非 const 方法 → `&mut self`，借用规则编译期保证。
2. **构造工厂**：默认构造 → `make_unique`，替代 `calculator_new` shim。
3. **Drop 析构**：hicc 自动析构，替代 `calculator_delete` shim。
4. **零手写**：工具默认支架即完整可用。
