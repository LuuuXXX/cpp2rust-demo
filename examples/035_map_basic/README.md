# 035_map_basic - std::map / std::unordered_map（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **std::map** 与 **std::unordered_map** 基本操作的 FFI 处理方式。采用 idiomatic 命名空间风格（`map_basic_ns`），不再使用 extern-C 不透明指针 + `*_new`/`*_delete` + impl 间接层；`StringIntMap` 直接持有 `std::map<std::string, int>`，`Counter` 直接持有 `std::unordered_map<std::string, int>`。析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### map_basic.h

```cpp
namespace map_basic_ns {

class StringIntMap {
    std::map<std::string, int> data_;
public:
    StringIntMap() = default;
    void insert(const char* key, int value) { data_[key ? key : ""] = value; }
    int get(const char* key) const { /* 未命中返回 -1 */ }
    int contains(const char* key) const { /* 返回 1/0 */ }
    int size() const { return (int)data_.size(); }
    int erase(const char* key) { return (int)data_.erase(key ? key : ""); }
    void clear() { data_.clear(); }
    const char* first_key() const { /* 返回首个有序 key 的 c_str() */ }
};

class Counter {
    std::unordered_map<std::string, int> counts_;
    std::string last_;
public:
    Counter() = default;
    void add(const char* word) { /* 词频 +1 */ }
    int count(const char* word) const { /* 未命中返回 0 */ }
    int unique_words() const { return (int)counts_.size(); }
    const char* last_word() const { return last_.c_str(); }
    void clear() { counts_.clear(); last_.clear(); }
};

} // namespace map_basic_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "map_basic.h"
}

hicc::import_class! {
    #[cpp(class = "map_basic_ns::StringIntMap")]
    pub class StringIntMap {
        #[cpp(method = "void insert(const char* key, int value)")]
        pub fn insert(&mut self, key: *const i8, value: i32);
        #[cpp(method = "int get(const char* key) const")]
        pub fn get(&self, key: *const i8) -> i32;
        // contains / size / erase / clear / first_key 略

        pub fn new() -> Self { string_int_map_new() }
    }
}

// Counter 同理（add/count/unique_words/last_word/clear）

hicc::import_lib! {
    #![link_name = "map_basic"]

    #[cpp(func = "std::unique_ptr<map_basic_ns::StringIntMap> hicc::make_unique<map_basic_ns::StringIntMap>()")]
    pub fn string_int_map_new() -> StringIntMap;
    // counter_new 同理
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 容器持有 | `std::map` / `std::unordered_map` 成员 | hicc 绑定内部持有，对外透明 |
| 插入/更新 | `insert` / `operator[]` | `insert(*const i8, i32)` |
| 查找 | `find` / `count` | `get` / `contains` / `count` |
| 删除 | `erase` / `clear` | 同名方法 |
| 字符串返回 | `const char*` 指向对象内字符串 | Rust 侧用 `CStr::from_ptr` 读取 |
| 析构 | C++ 默认析构 | Rust `Drop` 自动触发 |

## 运行结果

```
=== 035_map_basic - std::map / std::unordered_map（hicc 直出）===

size=2 apple=7 banana?=1
missing=-1 first_key=apple
erase banana=1 size=1
after clear size=0

counter rust=2 cpp=1 unique=2 last=rust

Rust FFI: hicc 直接绑定持有 std::map / std::unordered_map 的类，析构由 Rust Drop 自动完成
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_string_int_map_insert_get_contains` | insert / get / contains / size |
| `smoke_string_int_map_overwrite_and_first_key` | 覆盖写入与 `std::map` 有序首键 |
| `smoke_string_int_map_erase_clear` | erase / clear |
| `smoke_counter_counts_words` | unordered_map 词频计数 |
| `smoke_counter_last_word_clear_and_anchor` | const char* 返回 / clear / anchor |

### 运行方式

```bash
cd examples/035_map_basic/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ `std::map` / `std::unordered_map` 可通过 hicc 直接绑定持有它们的类来表达，无需不透明指针 + impl 间接层
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
- insert/get/contains/erase/count 等操作语义与 C++ 一致，跨 FFI 返回字符串时只暴露 `const char*`
