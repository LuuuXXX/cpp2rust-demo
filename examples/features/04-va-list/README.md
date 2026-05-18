# 特性示例：va_list 参数 → unsafe 可变参数绑定

## 背景

C++ 函数的最后一个参数若为 `va_list`，cpp2rust-demo 会：

1. 将该参数从 Rust 参数列表中删除
2. 在参数列表末尾追加 `...`（Rust 的 C-ABI 可变参数标记）
3. 将函数标记为 `unsafe fn`（可变参数调用本身是 unsafe 的）

生成的绑定与 C 的 ABI 完全匹配。

## 源码文件

- `va_logger.hpp`：`namespace logger` 下包含带 `va_list` 最后参数的函数
- `entry.cpp`：翻译单元入口

## 运行步骤

```bash
cpp2rust-demo init --feature feat04 --link logger --no-link \
    -- clang -x c++ -fsyntax-only examples/features/04-va-list/entry.cpp

cpp2rust-demo merge --feature feat04
cat .cpp2rust/feat04/rust/src/lib.rs
```

## 预期生成结果

```rust
hicc::import_lib! {
    #![link_name = "logger"]

    // va_list 最后参数 → unsafe fn + 尾部 `...`
    unsafe fn log_message(level: i32, fmt: *const i8, ...);

    unsafe fn format_string(buf: *mut i8, buf_size: i32, fmt: *const i8, ...) -> i32;

    // 普通函数正常提取
    fn flush();
}
```

## 关键结论

| C++ 函数 | 提取结果 |
|---------|---------|
| `void log_message(int, const char*, va_list)` | ✅ `unsafe fn log_message(level: i32, fmt: *const i8, ...)` |
| `int format_string(char*, int, const char*, va_list)` | ✅ `unsafe fn format_string(..., ...)` |
| `void flush()` | ✅ 正常提取 |

> 注意：`va_list` 参数本身在 Rust 侧不可见，由运行时 ABI 隐式传递。
> 调用端需要使用 Rust 的 `std::ffi::VaList` 或从 C 侧构造好 `va_list` 再传入。
