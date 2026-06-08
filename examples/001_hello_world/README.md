# 001_hello_world - 简单函数导出

## C++ 特性

本示例展示最基本的 C++ 函数导出，通过 FFI 供 Rust 调用。

## C++ 代码

### hello_world.h

```cpp
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

void hello_world(void);

#ifdef __cplusplus
}
#endif
```

关键点：
- `extern "C"` 确保 C++ 编译器使用 C 调用约定
- `void hello_world(void)` 是 C 风格函数签名

### hello_world.cpp

```cpp
#include "hello_world.h"
#include <iostream>

void hello_world(void) {
    std::cout << "Hello, World!" << std::endl;
}
```

## Rust FFI 代码

```rust
hicc::cpp! {
    #include "hello_world.h"
}

hicc::import_lib! {
    #![link_name = "hello_world"]

    #[cpp(func = "void hello_world()")]
    fn hello_world();
}
```
## 构建方法

### C++ 编译

```bash
cd cpp
g++ -c hello_world.cpp -o hello_world.o
g++ -shared -fPIC hello_world.cpp -o libhello_world.so  # Linux
# 或
g++ -shared -fPIC hello_world.cpp -o hello_world.dll    # Windows
```

### Rust 编译

```bash
cd rust_hicc
cargo build
```

## 运行结果

```
Hello, World!
Rust FFI: hello_world() called successfully!
```

## FFI 对比分析

| 方面 | C++ | Rust |
|------|-----|------|
| 函数声明 | `void hello_world(void)` | `#[cpp(func = "void hello_world(void)")]` |
| 符号链接 | 通过链接器 | `#[link_name = "..."]` |
| 调用方式 | 直接调用 | 包装为 Rust 函数 |

## 总结

本示例是最简单的 C++ FFI 场景：
- 纯 C 接口，无 C++ 特性的使用
- 函数签名保持 C 风格
- 通过 extern "C" 确保 ABI 兼容
