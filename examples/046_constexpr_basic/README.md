# 046_constexpr_basic - constexpr（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **constexpr** 构造函数、内联方法与模板函数的 FFI 处理方式。采用 idiomatic 命名空间风格（`constexpr_basic_ns`），不再使用 extern-C 不透明指针 + `*_new`/`*_delete` + impl 间接层；`ConstexprPoint` 直接持有 `int` 坐标。析构由 Rust 的 `Drop` 自动完成。

## C++ 代码

### constexpr_basic.h

```cpp
namespace constexpr_basic_ns {

class ConstexprPoint {
    int x_;
    int y_;
public:
    constexpr ConstexprPoint(int x, int y) : x_(x), y_(y) {}
    constexpr int x() const { return x_; }
    constexpr int y() const { return y_; }
    constexpr int manhattan_distance() const { return (x_<0?-x_:x_) + (y_<0?-y_:y_); }
};

template<int N> constexpr int fibonacci() { return fibonacci<N-1>() + fibonacci<N-2>(); }
template<> constexpr int fibonacci<0>() { return 0; }
template<> constexpr int fibonacci<1>() { return 1; }

int fibonacci_10();
int array_size();

} // namespace constexpr_basic_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定类、`make_unique` 工厂与命名空间自由函数：

```rust
hicc::cpp! {
    #include "constexpr_basic.h"
}

hicc::import_class! {
    #[cpp(class = "constexpr_basic_ns::ConstexprPoint")]
    pub class ConstexprPoint {
        #[cpp(method = "int x() const")]
        pub fn x(&self) -> i32;
        #[cpp(method = "int y() const")]
        pub fn y(&self) -> i32;
        #[cpp(method = "int manhattan_distance() const")]
        pub fn manhattan_distance(&self) -> i32;

        pub fn new(x: i32, y: i32) -> Self { constexpr_point_new(x, y) }
    }
}

hicc::import_lib! {
    #![link_name = "constexpr_basic"]

    #[cpp(func = "std::unique_ptr<constexpr_basic_ns::ConstexprPoint> hicc::make_unique<constexpr_basic_ns::ConstexprPoint, int, int>(int&&, int&&)")]
    pub fn constexpr_point_new(x: i32, y: i32) -> ConstexprPoint;

    #[cpp(func = "int constexpr_basic_ns::fibonacci_10()")]
    pub fn fibonacci_10() -> i32;
}
```

## FFI 对比分析

| 方面 | C++ constexpr | Rust FFI |
|------|---------------|----------|
| 类型持有 | `int` 坐标成员 | hicc 绑定内部持有，对外透明 |
| 构造 | `constexpr ConstexprPoint(int, int)` | `make_unique` 工厂 |
| 访问 | `constexpr` 内联方法 | 同名方法返回 i32 |
| 模板 | `fibonacci<10>()` | 命名空间自由函数返回标量 |
| 析构 | C++ 默认析构 | Rust `Drop` 自动触发 |

## 运行结果

```
=== 046_constexpr_basic - constexpr（hicc 直出）===

p x=3 y=4 manhattan=7
neg manhattan=7
fibonacci<10>()=55 array_size=16

Rust FFI: hicc 直接绑定 constexpr 类与命名空间自由函数，析构由 Rust Drop 自动完成
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_constexpr_point_positive_is_per_object` | x / y / manhattan_distance 的对象内状态 |
| `smoke_constexpr_point_negative_is_per_object` | 负坐标 manhattan_distance |
| `smoke_constexpr_free_functions` | fibonacci_10 / array_size |

### 运行方式

```bash
cd examples/046_constexpr_basic/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ `constexpr` 类可通过 hicc 直接绑定持有它的类来表达，无需不透明指针 + impl 间接层
- 构造经 `make_unique` 工厂，析构由 Rust `Drop` 自动完成，无需 `*_delete` shim
- 跨 FFI 只交换 `int` 等标量，模板与 constexpr 计算保留在 C++ 内部
