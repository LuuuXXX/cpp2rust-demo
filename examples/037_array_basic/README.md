# 037_array_basic - std::array（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **std::array** 基本操作的 FFI 处理方式。采用 idiomatic 命名空间风格
（`array_basic_ns`），不再使用 extern-C 不透明指针 + `*_new`/`*_delete` + impl 间接层；
`IntArray` 直接持有固定大小 `std::array<int, 8>`，演示 size/set/get/fill/sum/max/min
等操作。析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### array_basic.h

```cpp
namespace array_basic_ns {

class IntArray {
    std::array<int, 8> data_;
public:
    IntArray() : data_{} {}
    int size() const { return 8; }
    void set(int i, int v) { /* 边界检查 */ }
    int get(int i) const { /* 越界返回 0 */ }
    void fill(int v) { data_.fill(v); }
    int sum() const { /* 累加 */ }
    int max() const { /* 最大值 */ }
    int min() const { /* 最小值 */ }
};

} // namespace array_basic_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "array_basic.h"
}

hicc::import_class! {
    #[cpp(class = "array_basic_ns::IntArray")]
    pub class IntArray {
        #[cpp(method = "void set(int i, int v)")]
        pub fn set(&mut self, i: i32, v: i32);
        #[cpp(method = "int get(int i) const")]
        pub fn get(&self, i: i32) -> i32;
        // size / fill / sum / max / min 略

        pub fn new() -> Self { int_array_new() }
    }
}

hicc::import_lib! {
    #![link_name = "array_basic"]

    #[cpp(func = "std::unique_ptr<array_basic_ns::IntArray> hicc::make_unique<array_basic_ns::IntArray>()")]
    pub fn int_array_new() -> IntArray;
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 容器持有 | `std::array<int, 8>` 成员 | hicc 绑定内部持有，对外透明 |
| 大小 | 编译期固定为 8 | `size()` 返回 i32 |
| 访问 | `operator[]` | `get` / `set`（带边界检查） |
| 批量赋值 | `fill` | 同名方法 |
| 析构 | `~IntArray` | Rust `Drop` 自动触发 |

## 运行结果

```
=== 037_array_basic - std::array（hicc 直出）===

size=8 sum=0
after set sum=280 min=0 max=70
get(2)=999 get(99)=0
after fill sum=56 min=7 max=7

Rust FFI: hicc 直接绑定持有 std::array 的类，析构由 Rust Drop 自动完成
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_int_array_size_and_zero_init` | size / 零初始化 / sum / min / max |
| `smoke_int_array_set_get_sum` | set / get / sum |
| `smoke_int_array_fill_min_max` | fill / min / max |
| `smoke_int_array_oob_is_safe` | 越界 get 返回 0，越界 set 不修改状态 |
| `smoke_int_array_per_object_state` | 每个对象独立保存状态 |

### 运行方式

```bash
cd examples/037_array_basic/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ `std::array` 可通过 hicc 直接绑定持有它的类来表达，无需不透明指针 + impl 间接层
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
- get/set/fill/sum/min/max 等操作语义与 C++ 一致，越界访问保持安全默认值
