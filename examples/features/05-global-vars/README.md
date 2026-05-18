# 特性示例：全局变量绑定

## 背景

C++ 的命名空间全局变量可以通过 hicc 的 `#[cpp(data = "...")]` 属性从 Rust 侧访问。
cpp2rust-demo 会从头文件中提取 `extern` 声明，并在 `import_lib!` 中自动生成绑定：

- **可变全局变量** → `fn var_name() -> &'static mut T`（允许读写）
- **const 全局变量** → `fn var_name() -> &'static T`（只读）
- 全局变量名自动转换为 snake_case（e.g. `gMaxSize` → `g_max_size`）

## 源码文件

- `global_vars.hpp`：`namespace metrics` 下的若干全局变量声明
- `entry.cpp`：翻译单元入口

## 运行步骤

```bash
cpp2rust-demo init --feature feat05 --link metrics \
    -- clang -x c++ -fsyntax-only examples/features/05-global-vars/entry.cpp

cpp2rust-demo merge --feature feat05
cat .cpp2rust/feat05/rust/src/lib.rs
```

## 预期生成结果

```rust
hicc::import_lib! {
    #![link_name = "metrics"]

    // 可变全局变量 → &'static mut T
    #[cpp(data = "metrics::g_request_count")]
    fn g_request_count() -> &'static mut i32;

    // const 全局变量 → &'static T（只读）
    #[cpp(data = "metrics::g_max_latency_ms")]
    fn g_max_latency_ms() -> &'static f64;

    // 命名空间外的全局变量
    #[cpp(data = "metrics::g_debug_enabled")]
    fn g_debug_enabled() -> &'static mut bool;
}
```

## 接口报告

`meta/init-interface-report.md` 中会出现 `## Global Variables` 部分，
列出所有提取到的全局变量及其类型。

## 关键结论

| C++ 声明 | Rust 绑定 |
|---------|---------|
| `extern int g_request_count` | `fn g_request_count() -> &'static mut i32` |
| `extern const double g_max_latency_ms` | `fn g_max_latency_ms() -> &'static f64` |
| `extern bool g_debug_enabled` | `fn g_debug_enabled() -> &'static mut bool` |

> 注意：Rust 侧通过函数调用获取静态引用，解引用（写入可变变量）需要 `unsafe` 块。
