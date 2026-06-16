# 043_namespace_nested - 嵌套命名空间（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **嵌套命名空间** 的 FFI 处理方式。保持 idiomatic 嵌套命名空间
（`foo::bar::config` 与 `foo::baz`），不再使用 extern-C 不透明指针 + `*_new`/`*_delete`
+ impl 间接层；`ConfigManager` 直接持有 `std::map<std::string, int>`，`DataProcessor`
直接持有普通成员。析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### namespace_nested.h

```cpp
namespace foo {
namespace bar { namespace config {

class ConfigManager {
    std::map<std::string, int> values_;
public:
    ConfigManager() = default;
    void set_value(const char* key, int value) { values_[key ? key : ""] = value; }
    int get_value(const char* key) const { /* 未命中返回 -1 */ }
    int size() const { return (int)values_.size(); }
};

}}

namespace baz {
class DataProcessor {
    int multiplier_;
public:
    DataProcessor();
    int process(int input) const { return input * multiplier_; }
};
}

const char* get_version();
int get_build_number();

} // namespace foo
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定嵌套命名空间类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "namespace_nested.h"
}

hicc::import_class! {
    #[cpp(class = "foo::bar::config::ConfigManager")]
    pub class ConfigManager {
        #[cpp(method = "void set_value(const char* key, int value)")]
        pub fn set_value(&mut self, key: *const i8, value: i32);
        #[cpp(method = "int get_value(const char* key) const")]
        pub fn get_value(&self, key: *const i8) -> i32;
        // size 略

        pub fn new() -> Self { config_manager_new() }
    }
}

// DataProcessor 同理（foo::baz::DataProcessor）

hicc::import_lib! {
    #![link_name = "namespace_nested"]

    #[cpp(func = "std::unique_ptr<foo::bar::config::ConfigManager> hicc::make_unique<foo::bar::config::ConfigManager>()")]
    pub fn config_manager_new() -> ConfigManager;
    // data_processor_new / get_version / get_build_number 同理
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 命名空间 | `foo::bar::config` / `foo::baz` | `#[cpp(class = "...")]` 使用完全限定名 |
| 容器持有 | `std::map<std::string, int>` 成员 | hicc 绑定内部持有，对外透明 |
| 查找 | `std::map::find` | `get_value(*const i8)`，未命中返回 -1 |
| 普通类方法 | `DataProcessor::process` | 同名方法 |
| 析构 | C++ 默认析构 | Rust `Drop` 自动触发 |

## 运行结果

```
=== 043_namespace_nested - 嵌套命名空间（hicc 直出）===

size=3 timeout=30 retry=3
missing=-1
process(5)=15
version=1.0.0 build_number=42

Rust FFI: hicc 直接绑定嵌套命名空间类，析构由 Rust Drop 自动完成
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_config_manager_set_get_size` | set_value / get_value / size / 缺失键 |
| `smoke_config_manager_overwrite` | 覆盖写入与 size 不变 |
| `smoke_data_processor` | `process(5) == 15` |
| `smoke_top_level_functions` | version / build_number / anchor |

### 运行方式

```bash
cd examples/043_namespace_nested/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ 嵌套命名空间类可通过 hicc 完全限定名直接绑定，无需 flatten 成单一 `_ns`
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
- `ConfigManager` 直接持有 `std::map`，`DataProcessor` 保持 `foo::baz` 命名空间语义
