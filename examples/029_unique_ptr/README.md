# 029_unique_ptr - std::unique_ptr（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **独占所有权**（`std::unique_ptr` 语义）的 FFI 处理方式。采用 idiomatic
命名空间风格（`unique_ptr_ns`），不再使用 extern-C 不透明指针 + `*_new`/`*_delete` 桥接；
hicc 直出用 `std::unique_ptr` 持有对象所有权，析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### unique_ptr.h

```cpp
namespace unique_ptr_ns {

// 独占持有一段缓冲区
class UniqueBuffer {
    std::string data_;
public:
    explicit UniqueBuffer(int sz) : data_(sz, '\0') {}
    int size() const { return static_cast<int>(data_.size()); }
    char* data() { return &data_[0]; }
    void fill(char c) { for (auto& ch : data_) ch = c; }
    char at(int i) const { return data_[i]; }
    int use_count() const { return 1; } // 独占：恒为 1
};

class Processor {
    std::string buffer_;
public:
    Processor() = default;
    const char* process(const char* input);
};

} // namespace unique_ptr_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "unique_ptr.h"
}

hicc::import_class! {
    #[cpp(class = "unique_ptr_ns::UniqueBuffer")]
    pub class UniqueBuffer {
        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;
        #[cpp(method = "void fill(char c)")]
        pub fn fill(&mut self, c: i8);
        #[cpp(method = "char at(int i) const")]
        pub fn at(&self, i: i32) -> i8;
        #[cpp(method = "int use_count() const")]
        pub fn use_count(&self) -> i32;
        // data() 略

        pub fn new(sz: i32) -> Self { unique_buffer_new(sz) }
    }
}

// Processor 同理

hicc::import_lib! {
    #![link_name = "unique_ptr"]

    #[cpp(func = "std::unique_ptr<unique_ptr_ns::UniqueBuffer> hicc::make_unique<unique_ptr_ns::UniqueBuffer, int>(int&&)")]
    pub fn unique_buffer_new(sz: i32) -> UniqueBuffer;
    // processor_new 同理
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 所有权 | `std::unique_ptr<T>` 独占 | hicc `unique_ptr` 包装，相当于 `Box<T>` |
| 构造 | `UniqueBuffer(sz)` | `UniqueBuffer::new(i32)`（`make_unique`） |
| 析构 | 作用域结束自动 | Rust `Drop` 自动触发 |
| use_count | 恒为 1 | `use_count()` 返回 1 |

## 运行结果

```
=== 029_unique_ptr - std::unique_ptr（hicc 直出）===

Buffer size: 16
Buffer data: AAAAAAAAAAAAAAAA
Use count: 1 (unique_ptr 恒为 1)

Processed result: Hello, unique_ptr! [processed]

Rust FFI: hicc 用 unique_ptr 管理 C++ 对象所有权
析构由 Rust Drop 自动触发，相当于 Box<T>
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_unique_buffer_new` | 容量正确 |
| `smoke_unique_buffer_use_count` | use_count 恒为 1 |
| `smoke_unique_buffer_fill_at` | fill/at 行为 |
| `smoke_unique_buffer_data` | data() 非空 |
| `smoke_processor_process` | process 返回含 [processed] |

### 运行方式

```bash
cd examples/029_unique_ptr/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ 独占所有权可通过 hicc 的 `unique_ptr` 包装直接表达
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
- 语义上相当于 Rust 的 `Box<T>`
