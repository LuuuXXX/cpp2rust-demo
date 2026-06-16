# 032_placement_new - 定位 new（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **定位 new**（placement new）的 FFI 处理方式。采用 idiomatic 命名空间风格
（`placement_new_ns`），不再使用 extern-C 不透明指针 + `void*` + `*_new`/`*_delete` 桥接；
`Buffer` 在预分配存储的指定偏移处用 placement new 构造 `SimpleValue`，`ObjectArray` 以
元素槽位逐个构造，模拟 `std::vector` 底层内存管理。存储由 Rust 的 `Drop` 自动回收。

## C++ 代码

### placement_new.h

```cpp
namespace placement_new_ns {

struct SimpleValue { int value; };

class Buffer {
    std::vector<char> storage_;
    std::size_t constructed_size_;
public:
    explicit Buffer(int capacity)
        : storage_(capacity > 0 ? (std::size_t)capacity : 0, 0), constructed_size_(0) {}
    int capacity() const { return (int)storage_.size(); }
    int size() const { return (int)constructed_size_; }
    int construct_at(int offset, int v) {           // placement new
        SimpleValue* p = new (storage_.data() + offset) SimpleValue{v};
        constructed_size_ = offset + sizeof(SimpleValue);
        return p->value;
    }
    int value_at(int offset) const { /* 读回 */ }
};

class ObjectArray {                                  // 模拟 vector 底层内存
    std::vector<char> storage_;
    int count_;
public:
    explicit ObjectArray(int count) : storage_(count * sizeof(SimpleValue), 0), count_(count) {}
    int count() const { return count_; }
    int element_size() const { return (int)sizeof(SimpleValue); }
    int emplace(int i, int v) {                      // 第 i 槽位 placement new
        SimpleValue* p = new (storage_.data() + i * sizeof(SimpleValue)) SimpleValue{v};
        return p->value;
    }
    int at(int i) const { /* 读回 */ }
};

} // namespace placement_new_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "placement_new.h"
}

hicc::import_class! {
    #[cpp(class = "placement_new_ns::Buffer")]
    pub class Buffer {
        #[cpp(method = "int construct_at(int offset, int v)")]
        pub fn construct_at(&mut self, offset: i32, v: i32) -> i32;
        #[cpp(method = "int value_at(int offset) const")]
        pub fn value_at(&self, offset: i32) -> i32;
        // capacity / size 略

        pub fn new(capacity: i32) -> Self { buffer_new(capacity) }
    }
}

// ObjectArray 同理（emplace / at / count / element_size）

hicc::import_lib! {
    #![link_name = "placement_new"]

    #[cpp(func = "std::unique_ptr<placement_new_ns::Buffer> hicc::make_unique<placement_new_ns::Buffer, int>(int&&)")]
    pub fn buffer_new(capacity: i32) -> Buffer;
    // object_array_new 同理
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 预分配存储 | `std::vector<char>` | hicc 绑定内部持有，对外透明 |
| 定位构造 | `new (ptr) SimpleValue{v}` | `construct_at` / `emplace` 返回读回值 |
| 读回 | `reinterpret_cast<SimpleValue*>` | `value_at` / `at` |
| 析构 | `~Buffer` 释放存储 | Rust `Drop` 自动触发 |

## 运行结果

```
=== 032_placement_new - 定位 new（hicc 直出）===

capacity=64
construct_at(0,42)=42 value_at(0)=42 size=4
construct_at(8,7)=7 value_at(8)=7

count=3 element_size=4
at(0)=10 at(1)=20 at(2)=30

Rust FFI: hicc 绑定在预分配存储中用 placement new 构造对象的类
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_buffer_capacity` | capacity() 返回正确 |
| `smoke_buffer_construct_and_read` | construct_at 后能读回值 |
| `smoke_buffer_out_of_range` | 越界 offset 返回 -1 |
| `smoke_object_array_emplace` | 逐槽位构造并读回 |
| `smoke_object_array_out_of_range` | 越界索引返回 -1 |

### 运行方式

```bash
cd examples/032_placement_new/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ 定位 new 可通过 hicc 绑定内部管理原始存储的类来表达
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim 或 `void*` 暴露
- placement new 在预分配存储中按偏移/槽位构造对象，行为可由 `value_at` / `at` 观测
