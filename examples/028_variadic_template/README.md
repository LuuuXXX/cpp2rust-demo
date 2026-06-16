# 028_variadic_template - 可变参数模板（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **可变参数模板**的 FFI 处理方式。函数模板 `sum<Args...>`（C++17 折叠
表达式）可对任意个数/类型的实参求和，但每个具体的「实参个数 + 类型」组合是一次独立
实例化，无法整体绑定到 Rust。本示例采用 idiomatic 命名空间风格（`variadic_template_ns`），
不再使用 extern-C 单态化 shim；各具体组合由 `rust_hicc/src/lib.rs` 中的 `hicc::cpp!`
命名包装函数补全。

## C++ 代码

### variadic_template.h

```cpp
namespace variadic_template_ns {

// 可变参数函数模板：二元右折叠对任意个数实参求和（0 个实参结果为 0）。
template <typename... Args>
auto sum(Args... args) {
    return (args + ... + 0);
}

// 锚点：本单元可链接的非模板符号。
int variadic_template_anchor();

} // namespace variadic_template_ns
```

## Rust FFI 代码

hicc 直出仅绑定可链接的非模板锚点 `variadic_template_anchor()`（见
`rust_hicc/src/lib_scaffold.rs`）。各「实参个数 + 类型」组合在手写 `lib.rs` 中补全：
每个 `hicc::cpp!` 包装函数显式实例化模板，再绑定为普通自由函数。

```rust
hicc::cpp! {
    #include "variadic_template.h"

    using namespace variadic_template_ns;

    // 显式实例化：固定「实参个数 + 类型」即一次模板实例化。
    int sum_i32_0() { return sum(); }
    int sum_i32_2(int a, int b) { return sum(a, b); }
    int sum_i32_3(int a, int b, int c) { return sum(a, b, c); }
    int sum_i32_5(int a, int b, int c, int d, int e) { return sum(a, b, c, d, e); }
    double sum_f64_2(double a, double b) { return sum(a, b); }
    double sum_f64_3(double a, double b, double c) { return sum(a, b, c); }
}

hicc::import_lib! {
    #![link_name = "variadic_template"]

    #[cpp(func = "int sum_i32_3(int, int, int)")]
    pub fn sum_i32_3(a: i32, b: i32, c: i32) -> i32;
    // 其余实例化与 anchor 同理
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 模板定义 | `template<Args...> auto sum(Args...)` | 不可直接导出 |
| 实例化 | 按实参个数/类型自动生成 | 经 `hicc::cpp!` 包装显式实例化 |
| 符号 | `sum<int,int,int>` | `sum_i32_3` |
| 任意元数 | 折叠表达式 | 每个元数一个包装函数 |

## 运行结果

```
=== 028_variadic_template - 可变参数模板 ===

sum_i32_0() = 0
sum_i32_2(10, 20) = 30
sum_i32_3(1, 2, 3) = 6
sum_i32_5(1, 2, 3, 4, 5) = 15

sum_f64_2(1.5, 2.5) = 4
sum_f64_3(1.5, 2.5, 3.0) = 7

Rust FFI: 可变参数模板按「实参个数 + 类型」逐一实例化
每个组合（sum<int,int>、sum<double,double,double> …）是一个独立实例
anchor() = 0
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
可链接 C++ 模板实例化，且基本行为正确。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_sum_i32` | 0/2/3/5 个 int 实参求和 |
| `smoke_sum_f64` | 2/3 个 double 实参求和 |
| `smoke_anchor` | `variadic_template_anchor()` 返回 0 |

### 运行方式

```bash
cd examples/028_variadic_template/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ 可变参数模板无法直接 FFI 导出（每个元数/类型组合是独立实例化）
- 策略：为每种需要的「实参个数 + 类型」创建显式实例化的 `hicc::cpp!` 包装函数
- 折叠表达式 `(args + ... + 0)` 让模板对任意元数（含 0）通用
- 这是 FFI 处理可变参数模板的标准方法
