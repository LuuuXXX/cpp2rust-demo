# 022_mutable_member - mutable 成员（hicc 直出，无 shim）

## C++ 特性

本示例展示 **`mutable` 成员**的地道 C++ 命名空间类 `DataFetcher`：成员 `access_count_`
被 `mutable` 修饰，因而即便在 `const` 方法 `fetch() const` 中也可被修改（「逻辑常量、
物理可变」的内部可变性）。用 hicc 直出绑定，**无 opaque 指针、无 `extern "C"` 桥接、
无 `*_new`/`*_delete` shim**。

## C++ 代码（节选）

```cpp
namespace mutable_member_ns {

class DataFetcher {
public:
    explicit DataFetcher(int seed);
    int fetch() const;        // const 方法，但修改 mutable 成员
    int accessCount() const;
private:
    int seed_;
    mutable int access_count_; // mutable：const 方法中可修改
};

} // namespace mutable_member_ns
```

配套 `standalone.sh` / `Makefile` 两种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出）

hicc 直出把 `const` 方法映射为 `&self`：可变更新发生在 C++ 侧（受 `mutable` 约束），
Rust 侧以共享引用调用即可，无需 `&mut self`。

```rust
hicc::import_class! {
    #[cpp(class = "mutable_member_ns::DataFetcher")]
    pub class DataFetcher {
        #[cpp(method = "int fetch() const")]
        pub fn fetch(&self) -> i32;        // &self，但底层更新 mutable 计数

        #[cpp(method = "int accessCount() const")]
        pub fn access_count(&self) -> i32;

        pub fn new(seed: i32) -> Self { data_fetcher_new(seed) }
    }
}
```

## 关键点

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| `const` 方法 | `&self`（共享引用） |
| `mutable` 成员 | C++ 侧内部可变；Rust 侧无需 `&mut self` |
| `fetch() const` 修改计数 | 调用 `&self` 方法即触发 mutable 更新 |
| 构造 `DataFetcher(int)` | `make_unique` 工厂 → `new(i32)` |

## 构建方法

```bash
cd cpp && ./standalone.sh        # 纯 C++ 独立验证（或 make run）
cd rust_hicc && cargo test       # 行为级 smoke 断言
```

## 总结

1. **mutable 内部可变**：`const` 方法可修改 `mutable` 成员。
2. **映射为 &self**：hicc 直出按 const 限定符映射，mutable 更新留在 C++ 侧。
3. **去 shim**：无 `*_new`/`*_delete`、无 opaque 指针、无 `extern "C"` 桥接。
