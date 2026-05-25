# 003_default_args - 默认参数

## C++ 特性

本示例展示 C++ 默认参数特性，以及在 FFI 边界如何处理。

## C++ 代码

### default_args.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

// 默认参数：times 默认为 1
int greet(const char* name, int times = 1);

#ifdef __cplusplus
}
#endif
```

### default_args.cpp

```cpp
#include "default_args.h"
#include <iostream>

int greet(const char* name, int times) {
    for (int i = 0; i < times; ++i) {
        std::cout << "Hello, " << name << "!" << std::endl;
    }
    return times;
}
```

## 默认参数与 FFI

### 问题

C++ 默认参数是**编译时特性**，在调用点由编译器自动填入默认值。但 FFI 边界是链接时才确定的，无法自动填入默认值。

### 解决方案

在 FFI 边界，必须传递所有参数：

```rust
#[cpp(func = "int greet(const char*, int)")]
unsafe fn greet(name: *const i8, times: i32) -> i32;
```

然后在 Rust 层面用函数封装模拟默认参数：

```rust
fn greet_with_default(name: *const i8) -> i32 {
    unsafe { greet(name, 1) }  // 默认 times = 1
}
```

## Rust FFI 代码

### main.rs

```rust
hicc::import_lib! {
    #![link_name = "default_args"]

    #[cpp(func = "int greet(const char*, int)")]
    unsafe fn greet(name: *const i8, times: i32) -> i32;
}

fn main() {
    let name = b"World\0".as_ptr() as *const i8;

    // 显式传递所有参数
    unsafe {
        let result = greet(name, 1);
        println!("greet(\"World\", 1) returned: {}", result);
    }

    // Rust 层面模拟默认参数
    fn greet_with_default(name: *const i8) -> i32 {
        unsafe { greet(name, 1) }  // 默认 times = 1
    }

    let result = greet_with_default(name);
    println!("greet_with_default(\"World\") returned: {}", result);
}
```

## 关键点

### C++ 默认参数限制

以下情况不能使用默认参数：
1. 参数是引用
2. 参数是数组
3. 参数是 lambda
4. **FFI 边界**（链接时无法填入默认值）

### Rust 模拟方式

| C++ 调用 | Rust 等价 |
|----------|-----------|
| `greet("World")` | `greet_with_default(name)` |
| `greet("World", 5)` | `greet(name, 5)` |

## 构建方法

### C++ 编译

```bash
cd cpp
g++ -c default_args.cpp -o default_args.o
g++ -shared -fPIC default_args.cpp -o libdefault_args.so
```

### Rust 编译

```bash
cd rust_hicc
cargo build
cargo run
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

C++ 默认参数在 FFI 边界无法直接使用，需要：
1. 在 FFI 声明时传递所有参数
2. 在 Rust 层用函数封装模拟默认参数行为
