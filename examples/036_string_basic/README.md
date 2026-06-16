# 036_string_basic - std::string（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **std::string** 基本操作的 FFI 处理方式。采用 idiomatic 命名空间风格
（`string_basic_ns`），不再使用 extern-C 不透明指针 + `*_new`/`*_delete` + impl 间接层；
`MyString` 直接持有 `std::string`，演示 length/empty/append/at/c_str/compare/to_upper/find
等操作。析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### string_basic.h

```cpp
namespace string_basic_ns {

class MyString {
    std::string data_;
public:
    explicit MyString(const char* s) : data_(s ? s : "") {}
    int length() const { return (int)data_.length(); }
    int empty() const { return data_.empty() ? 1 : 0; }
    void append(const char* s) { if (s) data_ += s; }
    char at(int i) const { /* 边界检查 */ }
    const char* c_str() const { return data_.c_str(); }
    int compare(const char* other) const { return data_.compare(other ? other : ""); }
    void to_upper() { /* 原地大写转换 */ }
    int find(const char* sub) const { /* 返回下标或 -1 */ }
};

} // namespace string_basic_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "string_basic.h"
}

hicc::import_class! {
    #[cpp(class = "string_basic_ns::MyString")]
    pub class MyString {
        #[cpp(method = "void append(const char* s)")]
        pub fn append(&mut self, s: *const i8);
        #[cpp(method = "const char* c_str() const")]
        pub fn c_str(&self) -> *const i8;
        #[cpp(method = "int find(const char* sub) const")]
        pub fn find(&self, sub: *const i8) -> i32;
        // length / empty / at / compare / to_upper 略

        pub fn new(s: *const i8) -> Self { my_string_new(s) }
    }
}

hicc::import_lib! {
    #![link_name = "string_basic"]

    #[cpp(func = "std::unique_ptr<string_basic_ns::MyString> hicc::make_unique<string_basic_ns::MyString, const char*>(const char*&&)")]
    pub fn my_string_new(s: *const i8) -> MyString;
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 字符串持有 | `std::string` 成员 | hicc 绑定内部持有，对外透明 |
| 修改 | `append` / `to_upper` | 同名方法 |
| 访问 | `c_str` / `at` | `*const i8` + `CStr` / `i8` |
| 查询 | `length` / `empty` / `find` | 同名方法返回 i32 |
| 析构 | `~MyString` | Rust `Drop` 自动触发 |

## 运行结果

```
=== 036_string_basic - std::string（hicc 直出）===

empty=0 length=5
after append=hello, world length=12
at(1)=e at(99)=0
compare hello=7
find world=7 find missing=-1
to_upper=HELLO, WORLD

Rust FFI: hicc 直接绑定持有 std::string 的类，析构由 Rust Drop 自动完成
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_length_and_empty` | length / empty |
| `smoke_append_and_c_str` | append 后 c_str 内容 |
| `smoke_at_bounds` | at 边界检查 |
| `smoke_compare` | compare 相同和字典序结果 |
| `smoke_to_upper` | to_upper 后 c_str 内容 |
| `smoke_find` | find 返回下标或 -1 |

### 运行方式

```bash
cd examples/036_string_basic/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ `std::string` 可通过 hicc 直接绑定持有它的类来表达，无需不透明指针 + impl 间接层
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
- length/append/c_str/compare/to_upper/find 等操作语义与 C++ 一致
