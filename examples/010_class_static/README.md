# 010_class_static - 静态成员（hicc 直出，无 shim）

## C++ 特性

本示例展示带**静态成员变量 + 静态方法**的地道 C++ 命名空间类，用 hicc 直出方式绑定：
默认构造派生 `hicc::make_unique` 工厂，实例方法直出；静态方法以「全限定自由函数式」
绑定（`Counter::instance_count()`）。静态计数随实例构造/析构（hicc `Drop`）自动维护，
**无 opaque 指针、无 `extern "C"` 桥接**。

## C++ 代码（节选）

```cpp
namespace class_static_ns {

class Counter {
public:
    Counter();                              // 构造：++instance_count_
    ~Counter();                             // 析构：--instance_count_

    int value() const;
    void increment();

    static int instance_count();            // 静态：当前存活实例数
    static void reset_instance_count();      // 静态：计数清零

private:
    int value_;
    static int instance_count_;             // 跨实例共享
};

} // namespace class_static_ns
```

配套 `standalone.sh` / `Makefile` 两种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出）

```rust
hicc::import_class! {
    #[cpp(class = "class_static_ns::Counter")]
    pub class Counter {
        #[cpp(method = "int value() const")]
        pub fn value(&self) -> i32;

        #[cpp(method = "void increment()")]
        pub fn increment(&mut self);

        pub fn new() -> Self { counter_new() }
    }
}

hicc::import_lib! {
    #![link_name = "class_static"]

    #[cpp(func = "std::unique_ptr<class_static_ns::Counter> hicc::make_unique<class_static_ns::Counter>()")]
    pub fn counter_new() -> Counter;

    // 静态方法：全限定自由函数式调用。
    #[cpp(func = "int class_static_ns::Counter::instance_count()")]
    pub fn counter_instance_count() -> i32;

    #[cpp(func = "void class_static_ns::Counter::reset_instance_count()")]
    pub fn counter_reset_instance_count();
}
```

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| 默认构造 | `hicc::make_unique<T>` 工厂（`new`） |
| 实例方法 | `import_class!` 直出（`value`/`increment`） |
| 静态方法 | `import_lib!` 中「全限定自由函数式」绑定 `class_static_ns::Counter::instance_count()` |
| 静态计数 | 随实例构造/`Drop` 自动维护，无需手工管理 |

> 工具默认产物（`lib_scaffold.rs`）只默认生成实例方法与构造工厂；静态方法由手写
> `lib.rs` 用全限定自由函数式补全。

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言（用互斥锁串行化共享静态计数）
```

## 总结

1. **构造工厂**：默认构造 → `make_unique`，替代 `counter_new` shim。
2. **静态方法绑定**：全限定自由函数式，替代 `counter_getInstanceCount` / `counter_resetInstanceCount` shim。
3. **Drop 析构**：hicc 自动析构维护静态计数，替代 `counter_delete` shim。
4. **共享状态测试**：smoke 用 `Mutex` 串行化访问全局静态计数，避免并行干扰。
