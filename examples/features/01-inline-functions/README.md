# 特性示例：inline 函数

## 背景

`inline` 是 C++ 的链接器提示关键字，对 clang AST 的函数声明节点没有语义影响。
cpp2rust-demo 直接从 AST 中提取函数签名，**忽略 `inline` 关键字**，
因此 inline 函数与普通函数生成完全相同的绑定。

## 源码文件

- `inline_math.hpp`：`namespace math` 下包含若干 inline 与非 inline 函数
- `entry.cpp`：仅 `#include` 头文件，作为翻译单元入口

## 运行步骤

```bash
cpp2rust-demo init --feature feat01 --link math --no-link \
    -- clang -x c++ -fsyntax-only examples/features/01-inline-functions/entry.cpp

cpp2rust-demo merge --feature feat01
cat .cpp2rust/feat01/rust/src/lib.rs
```

## 预期生成结果

inline 函数与非 inline 函数均进入同一 `import_lib!` 块：

```rust
hicc::import_lib! {
    #![link_name = "math"]

    // inline 函数：与普通函数生成相同绑定
    #[cpp(func = "int math::add(int, int)")]
    fn add(a: i32, b: i32) -> i32;

    #[cpp(func = "double math::mul(double, double)")]
    fn mul(x: f64, y: f64) -> f64;

    // 非 inline 函数
    #[cpp(func = "int math::subtract(int, int)")]
    fn subtract(a: i32, b: i32) -> i32;

    // inline 重载（overload suffix _2）
    #[cpp(func = "int math::clamp(int, int, int)")]
    fn clamp(v: i32, lo: i32, hi: i32) -> i32;

    #[cpp(func = "double math::clamp(double, double, double)")]
    fn clamp_2(v: f64, lo: f64, hi: f64) -> f64;
}
```

## 关键结论

| C++ 声明 | 提取结果 |
|---------|---------|
| `inline int add(...)` | ✅ 正常提取，与非 inline 相同 |
| `inline double mul(...)` | ✅ 正常提取 |
| `int subtract(...)` | ✅ 正常提取 |
| `inline int clamp(...) / double clamp(...)` | ✅ 重载自动加后缀 `_2` |

> `inline` 关键字对工具透明，用户无需做任何特殊处理。
