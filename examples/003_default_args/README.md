# 003_default_args - 命名空间函数 + C++ 默认参数

## C++ 特性

命名空间内自由函数带 C++ 默认参数（`times = 1`）。默认参数是 C++ 源语言特性，
FFI 边界需显式传入全部实参；函数本身由 hicc `import_lib!` 直出绑定。

## C++ 代码

### default_args.h

```cpp
#pragma once

namespace default_args_ns {

int greet(const char* name, int times = 1);

} // namespace default_args_ns
```

## Rust FFI 代码

```rust
hicc::import_lib! {
    #![link_name = "default_args"]

    #[cpp(func = "int default_args_ns::greet(const char*, int)")]
    pub unsafe fn greet(name: *const i8, times: i32) -> i32;
}
```

## 构建方法

```bash
cd cpp && ./standalone.sh    # 独立 C++ 验证
cd rust_hicc && cargo run    # Rust 端运行
```

## 运行结果

```
Hello, World!
greet("World", 1) returned: 1
Hello, World!
greet_with_default("World") returned: 1

Rust FFI: Default args simulated in Rust!
```

## 总结

- 命名空间函数无需 `extern "C"`，以 `ns::fn()` 直出绑定
- C++ 默认参数仅在 C++ 端生效；Rust 侧需传全部实参，可在 Rust 层封装模拟默认值
