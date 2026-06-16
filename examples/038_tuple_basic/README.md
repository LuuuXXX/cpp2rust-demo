# 038_tuple_basic - std::tuple（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **std::tuple** 基本操作的 FFI 处理方式。采用 idiomatic 命名空间风格
（`tuple_basic_ns`），不再使用 extern-C 不透明指针 + `*_new`/`*_delete` + impl 间接层；
`Record` 直接持有 `std::tuple<int, double, std::string>`，演示 id/score/name 等操作。
析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### tuple_basic.h

```cpp
namespace tuple_basic_ns {

class Record {
    std::tuple<int, double, std::string> data_;
public:
    Record(int id, double score, const char* name);
    int id() const { return std::get<0>(data_); }
    double score() const { return std::get<1>(data_); }
    const char* name() const { return std::get<2>(data_).c_str(); }
    void set_id(int id) { std::get<0>(data_) = id; }
    void set_score(double score) { std::get<1>(data_) = score; }
};

} // namespace tuple_basic_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "tuple_basic.h"
}

hicc::import_class! {
    #[cpp(class = "tuple_basic_ns::Record")]
    pub class Record {
        #[cpp(method = "int id() const")]
        pub fn id(&self) -> i32;
        #[cpp(method = "double score() const")]
        pub fn score(&self) -> f64;
        #[cpp(method = "const char* name() const")]
        pub fn name(&self) -> *const i8;
        // set_id / set_score 略

        pub fn new(id: i32, score: f64, name: *const i8) -> Self { record_new(id, score, name) }
    }
}

hicc::import_lib! {
    #![link_name = "tuple_basic"]

    #[cpp(func = "std::unique_ptr<tuple_basic_ns::Record> hicc::make_unique<tuple_basic_ns::Record, int, double, const char*>(int&&, double&&, const char*&&)")]
    pub fn record_new(id: i32, score: f64, name: *const i8) -> Record;
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| tuple 持有 | `std::tuple<int, double, std::string>` 成员 | hicc 绑定内部持有，对外透明 |
| 访问 | `std::get<N>` | `id` / `score` / `name` 方法 |
| 修改 | `std::get<N>(data_) = value` | `set_id` / `set_score` 方法 |
| 字符串 | `std::string` 存储，`c_str()` 只读暴露 | `*const i8` + `CStr::from_ptr` |
| 析构 | `~Record` | Rust `Drop` 自动触发 |

## 运行结果

```
=== 038_tuple_basic - std::tuple（hicc 直出）===

id=42 score=98.5 name=alice
after set id=100 score=88.25 name=alice

Rust FFI: hicc 直接绑定持有 std::tuple 的类，析构由 Rust Drop 自动完成
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_record_new_and_getters` | 构造后 id / score / name 正确 |
| `smoke_record_set_id` | set_id 只修改 id |
| `smoke_record_set_score` | set_score 只修改 score |
| `smoke_record_per_object_state` | 多对象状态互不影响 |

### 运行方式

```bash
cd examples/038_tuple_basic/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ `std::tuple` 可通过 hicc 直接绑定持有它的类来表达，无需不透明指针 + impl 间接层
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
- `std::get<N>` 访问的标量与字符串指针可按方法直接暴露给 Rust
