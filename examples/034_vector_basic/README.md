# 034_vector_basic - std::vector（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **std::vector** 基本操作的 FFI 处理方式。采用 idiomatic 命名空间风格
（`vector_basic_ns`），不再使用 extern-C 不透明指针 + `*_new`/`*_delete` + impl 间接层；
`IntVector` / `StringVector` 直接持有 `std::vector`，演示 size/capacity/push_back/get/set
等操作。析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### vector_basic.h

```cpp
namespace vector_basic_ns {

class IntVector {
    std::vector<int> data_;
public:
    IntVector() = default;
    int size() const { return (int)data_.size(); }
    int capacity() const { return (int)data_.capacity(); }
    int empty() const { return data_.empty() ? 1 : 0; }
    void reserve(int n) { if (n > 0) data_.reserve(n); }
    void push_back(int v) { data_.push_back(v); }
    void pop_back() { if (!data_.empty()) data_.pop_back(); }
    int get(int i) const { /* 边界检查 */ }
    void set(int i, int v) { /* 边界检查 */ }
    int sum() const { /* 累加 */ }
    void clear() { data_.clear(); }
};

class StringVector {
    std::vector<std::string> data_;
public:
    int size() const { return (int)data_.size(); }
    void push_back(const char* s) { data_.push_back(s ? s : ""); }
    const char* get(int i) const { /* 返回元素 c_str() */ }
    void clear() { data_.clear(); }
};

} // namespace vector_basic_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "vector_basic.h"
}

hicc::import_class! {
    #[cpp(class = "vector_basic_ns::IntVector")]
    pub class IntVector {
        #[cpp(method = "void push_back(int v)")]
        pub fn push_back(&mut self, v: i32);
        #[cpp(method = "int get(int i) const")]
        pub fn get(&self, i: i32) -> i32;
        // size / capacity / set / sum / clear 略

        pub fn new() -> Self { int_vector_new() }
    }
}

// StringVector 同理（push_back(*const i8) / get -> *const i8）

hicc::import_lib! {
    #![link_name = "vector_basic"]

    #[cpp(func = "std::unique_ptr<vector_basic_ns::IntVector> hicc::make_unique<vector_basic_ns::IntVector>()")]
    pub fn int_vector_new() -> IntVector;
    // string_vector_new 同理
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 容器持有 | `std::vector<T>` 成员 | hicc 绑定内部持有，对外透明 |
| 增删 | `push_back` / `pop_back` | 同名方法 |
| 访问 | `operator[]` | `get` / `set`（带边界检查） |
| 容量 | `size` / `capacity` | 同名方法返回 i32 |
| 析构 | `~IntVector` | Rust `Drop` 自动触发 |

## 运行结果

```
=== 034_vector_basic - std::vector（hicc 直出）===

empty=1
size=5 sum=100
get(2)=999
after pop_back size=4
after clear empty=1

sv size=2 get(0)=alpha get(1)=beta

Rust FFI: hicc 直接绑定持有 std::vector 的类，析构由 Rust Drop 自动完成
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_int_vector_push_and_sum` | push_back / size / sum / empty |
| `smoke_int_vector_get_set` | get / set 往返 |
| `smoke_int_vector_pop_clear` | pop_back / clear |
| `smoke_int_vector_reserve_capacity` | reserve 后容量 >= 8 |
| `smoke_string_vector` | 字符串容器 push_back / get |

### 运行方式

```bash
cd examples/034_vector_basic/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ `std::vector` 可通过 hicc 直接绑定持有它的类来表达，无需不透明指针 + impl 间接层
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
- push_back/get/set/size/capacity 等操作语义与 C++ 一致
