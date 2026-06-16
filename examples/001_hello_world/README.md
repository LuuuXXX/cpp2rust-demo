# 001_hello_world - 命名空间自由函数导出

## C++ 特性

最基本的自由函数导出：函数置于命名空间中，无需 `extern "C"`，由 hicc
`import_lib!` 以 `ns::fn()` 形式直出绑定（去 shim）。

## C++ 代码

### hello_world.h

```cpp
#pragma once

namespace hello_world_ns {

// 命名空间内自由函数：无需 extern "C"，由 hicc import_lib! 以 ns::fn() 直出绑定。
void hello_world();

} // namespace hello_world_ns
```

### hello_world.cpp

```cpp
#include "hello_world.h"
#include <iostream>

namespace hello_world_ns {

void hello_world() {
    std::cout << "Hello, World!" << std::endl;
}

} // namespace hello_world_ns
```

## Rust FFI 代码

```rust
hicc::cpp! {
    #include "hello_world.h"
}

hicc::import_lib! {
    #![link_name = "hello_world"]

    #[cpp(func = "void hello_world_ns::hello_world()")]
    pub fn hello_world();
}
```

## 构建方法

### 独立 C++ 验证

```bash
cd cpp
./standalone.sh   # 或 make run
```

### Rust 编译运行

```bash
cd rust_hicc
cargo run
```

## 运行结果

```
Hello, World!
Rust FFI: hello_world() called successfully!
```

## 总结

- 自由函数置于命名空间，无需 `extern "C"`
- hicc 以 `#[cpp(func = "ns::fn()")]` 直接绑定真实的 C++ 命名空间函数
- 被绑定的函数需在实现单元（`.cpp`）内定义
