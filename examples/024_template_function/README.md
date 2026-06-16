# 024_template_function - 函数模板（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **函数模板**的 FFI 处理方式。模板是编译期多态：`template<T>` 只是
「蓝图」，每个实例化（`do_swap<int>`、`do_swap<double>` …）才是独立的具体函数。
模板本身没有可链接符号，无法整体绑定到 Rust，必须为每个需要的具体类型做实例化。

本示例采用 idiomatic 命名空间风格（`template_function_ns`），不再使用 extern-C 单态化
shim；各类型实例化由 `rust_hicc/src/lib.rs` 中的 `hicc::cpp!` 命名包装函数补全。

## C++ 代码

### template_function.h

```cpp
namespace template_function_ns {

// 函数模板：按指针交换两个对象。
template <typename T>
void do_swap(T* a, T* b) { T t = *a; *a = *b; *b = t; }

// 函数模板：返回两者中较大的值。
template <typename T>
T max_value(T a, T b) { return a > b ? a : b; }

// 锚点：本单元可链接的非模板符号。
int template_function_anchor();

} // namespace template_function_ns
```

## Rust FFI 代码

hicc 直出仅绑定可链接的非模板锚点 `template_function_anchor()`（见
`rust_hicc/src/lib_scaffold.rs`）。各类型实例化在手写 `lib.rs` 中补全：每个
`hicc::cpp!` 包装函数显式实例化模板，再绑定为普通自由函数。

```rust
hicc::cpp! {
    #include "template_function.h"

    using namespace template_function_ns;

    // 显式实例化：每个具体类型的包装即一次模板实例化。
    void swap_i32(int* a, int* b) { do_swap<int>(a, b); }
    void swap_f64(double* a, double* b) { do_swap<double>(a, b); }

    int max_i32(int a, int b) { return max_value<int>(a, b); }
    double max_f64(double a, double b) { return max_value<double>(a, b); }
}

hicc::import_lib! {
    #![link_name = "template_function"]

    #[cpp(func = "void swap_i32(int*, int*)")]
    pub unsafe fn swap_i32(a: *mut i32, b: *mut i32);

    #[cpp(func = "void swap_f64(double*, double*)")]
    pub unsafe fn swap_f64(a: *mut f64, b: *mut f64);

    #[cpp(func = "int max_i32(int, int)")]
    pub fn max_i32(a: i32, b: i32) -> i32;

    #[cpp(func = "double max_f64(double, double)")]
    pub fn max_f64(a: f64, b: f64) -> f64;

    #[cpp(func = "int template_function_anchor()")]
    pub fn template_function_anchor() -> i32;
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 模板定义 | `template<T> void do_swap(T*, T*)` | 不可直接导出 |
| 模板实例化 | 使用点自动生成 | 经 `hicc::cpp!` 包装显式实例化 |
| 符号 | `do_swap<int>` / `do_swap<double>` | `swap_i32` / `swap_f64` |
| 类型安全 | 编译器保证 | 包装函数按类型固定 |

## 运行结果

```
=== 024_template_function - 函数模板 ===

Before swap: a = 10, b = 20
After swap:  a = 20, b = 10

Before swap: x = 3.14, y = 2.71
After swap:  x = 2.71, y = 3.14

max_i32(3, 7) = 7
max_f64(2.5, 1.5) = 2.5

Rust FFI: 模板必须在 C++ 侧按具体类型实例化
每个实例化（do_swap<int>、do_swap<double> …）是一个独立的具体函数
anchor() = 0
```

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
可链接 C++ 模板实例化，且基本行为正确。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_swap_i32` | `swap_i32(&mut a, &mut b)` 后 a、b 互换 |
| `smoke_swap_f64` | `swap_f64(&mut x, &mut y)` 后 x、y 互换 |
| `smoke_max_value` | `max_i32` / `max_f64` 返回较大值 |
| `smoke_anchor` | `template_function_anchor()` 返回 0 |

### 运行方式

```bash
cd examples/024_template_function/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ 模板无法直接 FFI 导出（模板本身没有可链接符号）
- 策略：为每种需要的类型创建显式实例化的 `hicc::cpp!` 包装函数
- Rust 端调用时即对应一个具体类型的实例化
- 这是 FFI 处理模板的标准方法
