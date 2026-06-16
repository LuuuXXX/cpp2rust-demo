# 030_shared_ptr - std::shared_ptr（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **共享所有权**（`std::shared_ptr` 语义）的 FFI 处理方式。采用 idiomatic
命名空间风格（`shared_ptr_ns`），不再使用 extern-C 不透明指针 + `*_new`/`*_delete` 桥接；
`SharedData` 内部以 `std::shared_ptr` 持有负载，`Cache` 用 `vector<shared_ptr>` 缓存，
演示引用计数增长。析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### shared_ptr.h

```cpp
namespace shared_ptr_ns {

class SharedData {
    std::shared_ptr<std::string> data_;
public:
    explicit SharedData(const char* name)
        : data_(std::make_shared<std::string>(name ? name : "")) {}
    const char* name() const { return data_->c_str(); }
    int use_count() const { return (int)data_.use_count(); }
    void reset() { data_.reset(); }
    int expired() const { return data_ ? 0 : 1; }
};

class Cache {
    std::vector<std::shared_ptr<std::string>> entries_;
public:
    Cache() = default;
    int store(const char* name) {
        auto sp = std::make_shared<std::string>(name ? name : "");
        entries_.push_back(sp);
        return (int)sp.use_count(); // 本地 sp + 缓存副本 = 2
    }
    int size() const { return (int)entries_.size(); }
    void clear() { entries_.clear(); }
};

} // namespace shared_ptr_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "shared_ptr.h"
}

hicc::import_class! {
    #[cpp(class = "shared_ptr_ns::SharedData")]
    pub class SharedData {
        #[cpp(method = "const char* name() const")]
        pub fn name(&self) -> *const i8;
        #[cpp(method = "int use_count() const")]
        pub fn use_count(&self) -> i32;
        // reset / expired 略

        pub fn new(name: *const i8) -> Self { shared_data_new(name) }
    }
}

// Cache 同理（store / size / clear）

hicc::import_lib! {
    #![link_name = "shared_ptr"]

    #[cpp(func = "std::unique_ptr<shared_ptr_ns::SharedData> hicc::make_unique<shared_ptr_ns::SharedData, const char*>(const char*&&)")]
    pub fn shared_data_new(name: *const i8) -> SharedData;
    // cache_new 同理
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 共享所有权 | `std::shared_ptr<T>` 引用计数 | hicc 包装，相当于 `Arc<T>` |
| 构造 | `SharedData("x")` | `SharedData::new(*const i8)`（`make_unique`） |
| 引用计数 | `use_count()` | `use_count()` 返回 i32 |
| 释放 | `shared_ptr::reset()` | `reset()` + `expired()` |
| 析构 | 计数归零自动 | Rust `Drop` 自动触发 |

## 运行结果

```
=== 030_shared_ptr - std::shared_ptr（hicc 直出）===

name=TestData use_count=1 expired=0
after reset expired=1

store use_count=2,2 size=2
after clear size=0

Rust FFI: hicc 用 shared_ptr 表达共享所有权，相当于 Rust 的 Arc<T>
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_shared_data_name` | name() 返回正确 |
| `smoke_shared_data_use_count` | 独立对象计数为 1 |
| `smoke_shared_data_reset_expired` | reset 后 expired |
| `smoke_cache_store` | 缓存后计数为 2 |
| `smoke_cache_clear` | clear 清空 |

### 运行方式

```bash
cd examples/030_shared_ptr/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ 共享所有权可通过 hicc 绑定内部使用 `std::shared_ptr` 的类来表达
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
- 语义上相当于 Rust 的 `Arc<T>`
