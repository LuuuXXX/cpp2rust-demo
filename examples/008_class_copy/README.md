# 008_class_copy - 深拷贝构造（hicc 直出，无 shim）

## C++ 特性

本示例展示带**深拷贝构造函数**的地道 C++ 命名空间类，用 hicc 直出方式绑定：
默认构造与 `int` 构造各派生一条 `hicc::make_unique` 工厂，拷贝构造
`Buffer(const Buffer&)` 由手写 `lib.rs` 补全；析构交给 hicc 的 `Drop`，
**无 opaque 指针、无 `extern "C"` 桥接**。

## C++ 代码（节选）

```cpp
namespace class_copy_ns {

class Buffer {
public:
    Buffer();                       // 默认构造（空缓冲区）
    explicit Buffer(int sz);        // 指定大小构造
    Buffer(const Buffer& other);    // 深拷贝构造（独立内存）
    Buffer& operator=(const Buffer& other); // 深拷贝赋值（rule of three）
    ~Buffer();                      // 释放内存

    void set(int index, int value);
    int get(int index) const;
    int size() const;
};

} // namespace class_copy_ns
```

配套 `standalone.sh` / `Makefile` 两种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出）

```rust
hicc::import_class! {
    #[cpp(class = "class_copy_ns::Buffer")]
    pub class Buffer {
        #[cpp(method = "void set(int index, int value)")]
        pub fn set(&mut self, index: i32, value: i32);

        #[cpp(method = "int get(int index) const")]
        pub fn get(&self, index: i32) -> i32;

        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        pub fn new() -> Self { buffer_new() }
        pub fn new_2(sz: i32) -> Self { buffer_new_2(sz) }
        pub fn from_copy(other: &Buffer) -> Self { buffer_from_copy(other) }
    }
}

hicc::import_lib! {
    #![link_name = "class_copy"]

    #[cpp(func = "std::unique_ptr<class_copy_ns::Buffer> hicc::make_unique<class_copy_ns::Buffer>()")]
    pub fn buffer_new() -> Buffer;

    #[cpp(func = "std::unique_ptr<class_copy_ns::Buffer> hicc::make_unique<class_copy_ns::Buffer, int>(int&&)")]
    pub fn buffer_new_2(sz: i32) -> Buffer;

    #[cpp(func = "std::unique_ptr<class_copy_ns::Buffer> hicc::make_unique<class_copy_ns::Buffer, const class_copy_ns::Buffer&>(const class_copy_ns::Buffer&)")]
    pub fn buffer_from_copy(other: &Buffer) -> Buffer;
}
```

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| 默认 / `int` 构造 | 各派生一条 `hicc::make_unique<T, Args...>` 工厂（`new`/`new_2`） |
| 拷贝构造 `Buffer(const Buffer&)` | 默认支架排除拷贝/移动构造；手写 `make_unique<Buffer, const Buffer&>` 工厂（`from_copy`） |
| 深拷贝独立性 | 拷贝体与原对象内存独立，修改原对象不影响拷贝 |
| 析构函数 | 由 hicc `Drop` 自动负责，无需 `*_delete` |

> 工具默认产物（`lib_scaffold.rs`）只默认生成参数可直出映射的构造（`new`/`new_2`）与
> 基本类型方法；拷贝构造由手写 `lib.rs` 用 `const class_copy_ns::Buffer&` 补全。

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 运行结果

```
Buffer(int) ctor, size=5
Buffer(const Buffer&) copy ctor, size=5
b2 size: 5
b2 values: 10 20 30 40 50 
after b1[0]=999: b1[0]=999 b2[0]=10 (unchanged)
~Buffer() dtor, size=5
~Buffer() dtor, size=5
```

## 总结

1. **多工厂**：默认 / `int` 构造 → `make_unique` 工厂，替代 `buffer_new` / `buffer_newWithSize` shim。
2. **拷贝工厂**：手写 `make_unique<Buffer, const Buffer&>`，替代 `buffer_newCopy` shim。
3. **Drop 析构**：hicc 自动析构，替代 `buffer_delete` shim。
4. **深拷贝独立**：smoke 断言验证修改原对象后拷贝保持不变。
