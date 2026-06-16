# 031_custom_deleter - 自定义删除器（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **自定义删除器**（custom deleter）的 FFI 处理方式。采用 idiomatic 命名空间
风格（`custom_deleter_ns`），不再使用 extern-C 不透明指针 + 函数指针回调 + `*_delete` 桥接；
`ManagedResource` 内部以带自定义删除器的 `std::unique_ptr<std::string, LoggingDeleter>`
持有负载。析构由 Rust 的 `Drop` 自动完成，届时内部 `unique_ptr` 会调用自定义删除器。

## C++ 代码

### custom_deleter.h

```cpp
namespace custom_deleter_ns {

int cleanup_count(); // 自定义删除器被调用的累计次数

struct LoggingDeleter {
    void operator()(std::string* p) const; // 记录一次清理后 delete
};

class ManagedResource {
    std::unique_ptr<std::string, LoggingDeleter> res_;
public:
    explicit ManagedResource(const char* name)
        : res_(new std::string(name ? name : ""), LoggingDeleter{}) {}
    const char* name() const { return res_ ? res_->c_str() : ""; }
    int released() const { return res_ ? 0 : 1; }
    void release() { res_.reset(); } // 触发自定义删除器
};

} // namespace custom_deleter_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，也无需手写函数指针回调，直接绑定类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "custom_deleter.h"
}

hicc::import_class! {
    #[cpp(class = "custom_deleter_ns::ManagedResource")]
    pub class ManagedResource {
        #[cpp(method = "const char* name() const")]
        pub fn name(&self) -> *const i8;
        #[cpp(method = "int released() const")]
        pub fn released(&self) -> i32;
        #[cpp(method = "void release()")]
        pub fn release(&mut self);

        pub fn new(name: *const i8) -> Self { managed_resource_new(name) }
    }
}

hicc::import_lib! {
    #![link_name = "custom_deleter"]

    #[cpp(func = "std::unique_ptr<custom_deleter_ns::ManagedResource> hicc::make_unique<custom_deleter_ns::ManagedResource, const char*>(const char*&&)")]
    pub fn managed_resource_new(name: *const i8) -> ManagedResource;

    #[cpp(func = "int custom_deleter_ns::cleanup_count()")]
    pub fn cleanup_count() -> i32;
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 自定义删除策略 | `unique_ptr<T, Deleter>` | hicc 绑定内部持有，对外透明 |
| 构造 | `ManagedResource("x")` | `ManagedResource::new(*const i8)`（`make_unique`） |
| 主动释放 | `unique_ptr::reset()` | `release()` + `released()` |
| 删除器触发 | 析构/reset 自动调用 | Rust `Drop` 自动触发，`cleanup_count` 可观测 |

## 运行结果

```
=== 031_custom_deleter - 自定义删除器（hicc 直出）===

name=logfile.txt released=0
after release released=1
cleanup_count delta=1

Rust FFI: hicc 绑定内部使用 unique_ptr<T, Deleter> 的类，
自定义删除器在对象析构（Rust Drop）时被自动调用
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_managed_resource_name` | name() 返回正确 |
| `smoke_managed_resource_release` | release 后 released() 为 1 |
| `smoke_custom_deleter_invoked` | 析构时自定义删除器被调用（计数增长） |

### 运行方式

```bash
cd examples/031_custom_deleter/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ 自定义删除器可通过 hicc 绑定内部使用 `unique_ptr<T, Deleter>` 的类来表达
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim 或函数指针回调
- 删除器在对象析构时被自动调用，行为可由 `cleanup_count()` 观测
