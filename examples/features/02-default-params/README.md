# 特性示例：默认参数

## 背景

C++ 的默认参数（default arguments）在 clang AST 中可见，但 cpp2rust-demo 的绑定层
**始终提取完整参数列表**，并忽略默认值本身（因为 hicc FFI 层无法表达 C++ 默认参数语义）。

这意味着：
- 工具会提取函数的所有参数（包括有默认值的参数）
- Rust 侧调用时必须显式传所有参数
- 如果需要"可省略"行为，用户可在 Rust 侧自行封装

## 源码文件

- `default_params.hpp`：`namespace config` 下包含带默认参数的函数
- `entry.cpp`：翻译单元入口

## 运行步骤

```bash
cpp2rust-demo init --feature feat02 --link config --no-link \
    -- clang -x c++ -fsyntax-only examples/features/02-default-params/entry.cpp

cpp2rust-demo merge --feature feat02
cat .cpp2rust/feat02/rust/src/merged_ffi.rs
```

## 预期生成结果

所有参数均被提取，默认值被忽略：

```rust
hicc::import_lib! {
    #![link_name = "config"]

    #[cpp(func = "int config::get_timeout()")]
    fn get_timeout() -> i32;

    // 带默认参数 notify=false — 两个参数均提取，默认值丢弃
    #[cpp(func = "void config::set_timeout(int, bool)")]
    fn set_timeout(ms: i32, notify: bool);

    // t=0.5 的默认值被忽略
    #[cpp(func = "double config::lerp(double, double, double)")]
    fn lerp(a: f64, b: f64, t: f64) -> f64;

    // level=1 的默认值被忽略
    #[cpp(func = "void config::log(const char *, int)")]
    fn log(msg: *const i8, level: i32);
}
```

## 关键结论

| C++ 声明 | 提取行为 |
|---------|---------|
| `void set_timeout(int ms, bool notify = false)` | ✅ 两参数均提取，默认值忽略 |
| `double lerp(double a, double b, double t = 0.5)` | ✅ 三参数均提取 |
| `void log(const char* msg, int level = 1)` | ✅ 两参数均提取 |

> Rust 调用侧需显式传入所有参数。如需"可省略"语义，在 Rust 侧用包装函数实现。
