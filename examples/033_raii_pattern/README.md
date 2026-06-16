# 033_raii_pattern - RAII 资源管理（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **RAII**（Resource Acquisition Is Initialization）的 FFI 处理方式。采用
idiomatic 命名空间风格（`raii_pattern_ns`），不再使用 extern-C 不透明指针 +
`*_new`/`*_delete` + 手写 `lock`/`unlock` 桥接；`Resource` 构造时获取资源（活跃计数 +1）、
析构时释放（计数 -1），`Transaction` 是作用域守卫，未 `commit()` 即析构则自动回滚。
析构由 Rust 的 `Drop` 自动触发，RAII 语义与 C++ 完全一致。

## C++ 代码

### raii_pattern.h

```cpp
namespace raii_pattern_ns {

int active_count();    // 当前存活 Resource 数量
int rollback_count();  // 未提交即析构的 Transaction 累计数量

class Resource {
    std::string name_;
public:
    explicit Resource(const char* name); // ++active_count
    ~Resource();                          // --active_count
    const char* name() const { return name_.c_str(); }
};

class Transaction {
    bool committed_;
public:
    Transaction();   // committed_ = false
    ~Transaction();  // if (!committed_) ++rollback_count
    void commit() { committed_ = true; }
    int committed() const { return committed_ ? 1 : 0; }
};

} // namespace raii_pattern_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定类与 `make_unique` 工厂；析构由 Rust `Drop` 触发：

```rust
hicc::cpp! {
    #include "raii_pattern.h"
}

hicc::import_class! {
    #[cpp(class = "raii_pattern_ns::Resource")]
    pub class Resource {
        #[cpp(method = "const char* name() const")]
        pub fn name(&self) -> *const i8;

        pub fn new(name: *const i8) -> Self { resource_new(name) }
    }
}

// Transaction 同理（commit / committed）

hicc::import_lib! {
    #![link_name = "raii_pattern"]

    #[cpp(func = "std::unique_ptr<raii_pattern_ns::Resource> hicc::make_unique<raii_pattern_ns::Resource, const char*>(const char*&&)")]
    pub fn resource_new(name: *const i8) -> Resource;
    // transaction_new 同理

    #[cpp(func = "int raii_pattern_ns::active_count()")]
    pub fn active_count() -> i32;
    #[cpp(func = "int raii_pattern_ns::rollback_count()")]
    pub fn rollback_count() -> i32;
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 资源获取 | 构造函数 | `Resource::new()`（`make_unique`） |
| 资源释放 | 析构函数 | Rust `Drop` 自动触发 |
| 作用域守卫 | 析构时回滚 | Drop 时回滚，`rollback_count` 可观测 |
| 活跃资源观测 | `active_count()` | 同名 FFI 函数返回 i32 |

## 运行结果

```
=== 033_raii_pattern - RAII 资源管理（hicc 直出）===

active_count(start)=0
after acquire a: name=db active=1
after acquire b: active=2
after b released: active=1
after all released: active=0

t1 committed=1
t2 committed=0
rollback delta=1

Rust FFI: hicc 绑定 RAII 类，构造获取资源、Drop 自动释放
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_resource_raii` | name() 正确；构造 +1、析构恢复活跃计数 |
| `smoke_transaction_raii` | commit 后 committed()=1；未提交析构回滚、已提交不回滚 |

> 注：`active_count` 仅由 Resource 测试触达、`rollback_count` 仅由 Transaction 测试触达，
> 互不影响，故并行测试下断言仍确定。

### 运行方式

```bash
cd examples/033_raii_pattern/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ RAII 可通过 hicc 绑定「构造获取 / 析构释放」的类来表达
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
- 作用域守卫（未提交即回滚）等 RAII 习惯用法语义完全保留，行为可经计数函数观测
