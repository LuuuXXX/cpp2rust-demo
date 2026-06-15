# 009_class_move - 移动语义（hicc 直出，无 shim）

## C++ 特性

本示例展示带**移动构造 / 移动赋值**的地道 C++ 命名空间类，用 hicc 直出方式绑定：
默认构造与 `int` 构造各派生一条 `hicc::make_unique` 工厂；移动是 O(1) 资源转移
（窃取源对象指针并置空），经成员方法 `move_from` 暴露；析构交给 hicc 的 `Drop`，
**无 opaque 指针、无 `extern "C"` 桥接**。

## C++ 代码（节选）

```cpp
namespace class_move_ns {

class UniqueVector {
public:
    UniqueVector();                                  // 默认构造
    explicit UniqueVector(int size);                 // 指定大小构造
    UniqueVector(UniqueVector&& other) noexcept;     // 移动构造（窃取资源）
    UniqueVector& operator=(UniqueVector&& other) noexcept; // 移动赋值
    ~UniqueVector();                                 // 释放内存

    void set(int index, int value);
    int get(int index) const;
    int size() const;
    void move_from(UniqueVector& src);               // 从 src 移动资源
};

} // namespace class_move_ns
```

配套 `standalone.sh` / `Makefile` 两种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出）

```rust
hicc::import_class! {
    #[cpp(class = "class_move_ns::UniqueVector")]
    pub class UniqueVector {
        #[cpp(method = "void set(int index, int value)")]
        pub fn set(&mut self, index: i32, value: i32);

        #[cpp(method = "int get(int index) const")]
        pub fn get(&self, index: i32) -> i32;

        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        // 类引用参数补全命名空间限定，hicc 才能在 cpp! 展开处解析类型。
        #[cpp(method = "void move_from(class_move_ns::UniqueVector & src)")]
        pub fn move_from(&mut self, src: &mut UniqueVector);

        pub fn new() -> Self { unique_vector_new() }
        pub fn new_2(size: i32) -> Self { unique_vector_new_2(size) }
    }
}

hicc::import_lib! {
    #![link_name = "class_move"]

    #[cpp(func = "std::unique_ptr<class_move_ns::UniqueVector> hicc::make_unique<class_move_ns::UniqueVector>()")]
    pub fn unique_vector_new() -> UniqueVector;

    #[cpp(func = "std::unique_ptr<class_move_ns::UniqueVector> hicc::make_unique<class_move_ns::UniqueVector, int>(int&&)")]
    pub fn unique_vector_new_2(size: i32) -> UniqueVector;
}
```

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| 默认 / `int` 构造 | 各派生一条 `hicc::make_unique<T, Args...>` 工厂（`new`/`new_2`） |
| 移动构造 / 移动赋值 | C++ 内部 O(1) 资源转移语义，经成员方法 `move_from` 暴露 |
| 类引用参数 | 方法签名中裸类名补全命名空间限定（`class_move_ns::UniqueVector &`） |
| 析构函数 | 由 hicc `Drop` 自动负责，无需 `*_delete` |

> 工具默认产物（`lib_scaffold.rs`）与本示例 `lib.rs` 一致：移动语义经 `move_from`
> 成员方法直出，无需手写补全。

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 总结

1. **多工厂**：默认 / `int` 构造 → `make_unique` 工厂，替代 `unique_vector_new` / `unique_vector_newWithData` shim。
2. **移动转移**：`move_from` 把源资源移入自身并置空源，smoke 断言验证 size 转移与源置空。
3. **Drop 析构**：hicc 自动析构，替代 `unique_vector_delete` shim。
4. **命名空间限定**：类引用参数自动补全限定，确保 hicc 编译通过。
