# 006_class_basic - 基础类（hicc 直出，无 shim）

## C++ 特性

本示例展示如何用 **hicc 直出** 方式把地道的 C++ 命名空间类绑定到 Rust：
直接绑定真实类 `class_basic_ns::Counter`，构造走 `hicc::make_unique` 工厂，
析构交给 hicc 的 `Drop`，**不再编写任何 `extern "C"` 桥接函数或 opaque 指针**。

## C++ 代码

### class_basic.h

```cpp
#pragma once
#include <string>
#include <iostream>

namespace class_basic_ns {

class Counter {
public:
    Counter() : count_(0), name_("anon") {}
    explicit Counter(const std::string& name) : count_(0), name_(name) {}

    void inc() { ++count_; }
    void inc_by(int delta) { count_ += delta; }
    void reset() { count_ = 0; }

    int count() const { return count_; }
    const std::string& name() const { return name_; }

private:
    int count_;
    std::string name_;
};

} // namespace class_basic_ns
```

地道 C++：真实命名空间类、真实成员（含 `std::string`），无 opaque 指针、无桥接函数。
配套 `standalone.sh` / `Makefile` / `CMakeLists.txt` 三种纯 C++ 构建方式。

## Rust FFI 代码（hicc 直出）

```rust
hicc::cpp! {
    #include "class_basic.h"
    #include <hicc/std/string.hpp>
}

hicc::import_class! {
    class string = hicc_std::string;

    #[cpp(class = "class_basic_ns::Counter")]
    pub class Counter {
        #[cpp(method = "void inc()")]
        pub fn inc(&mut self);

        #[cpp(method = "void inc_by(int)")]
        pub fn inc_by(&mut self, delta: i32);

        #[cpp(method = "void reset()")]
        pub fn reset(&mut self);

        #[cpp(method = "int count() const")]
        pub fn count(&self) -> i32;

        #[cpp(method = "const std::string& name() const")]
        pub fn name(&self) -> &string;

        pub fn new() -> Self { counter_new() }
        pub fn with_name(name: &string) -> Self { counter_with_name(name) }
    }
}

hicc::import_lib! {
    #![link_name = "class_basic"]

    #[cpp(func = "std::unique_ptr<class_basic_ns::Counter> hicc::make_unique<class_basic_ns::Counter>()")]
    pub fn counter_new() -> Counter;

    #[cpp(func = "std::unique_ptr<class_basic_ns::Counter> hicc::make_unique<class_basic_ns::Counter, const std::string&>(const std::string&)")]
    pub fn counter_with_name(name: &hicc_std::string) -> Counter;
}
```

## 关键点

### C++ 类到 hicc 直出的映射

| C++ 概念 | hicc 直出映射 |
|----------|--------------|
| 命名空间类 | `#[cpp(class = "class_basic_ns::Counter")]` 直接绑定真实类 |
| 公有构造函数 | `import_lib!` 中 `hicc::make_unique<T, Args...>` 工厂，`import_class!` body 内 `pub fn new(...) -> Self { factory(...) }` 转发 |
| 析构函数 | 由 hicc 的 `Drop` 自动负责，**无需 `destroy =` 或 `*_delete`** |
| const 成员函数 | `fn count(&self) -> i32`（`&self`） |
| 非 const 成员函数 | `fn inc(&mut self)`（`&mut self`） |
| 返回 `const std::string&` | `fn name(&self) -> &string`，配 `class string = hicc_std::string;`（`ClassRef`） |

### 与旧版（C ABI shim）的区别

- 旧版用 `struct Counter` opaque 指针 + `counter_new`/`counter_delete`/`counter_get` 等
  手写 `extern "C"` 桥接；本版**全部删除**，直接绑定真实命名空间类。
- 构造由 `hicc::make_unique` 工厂直出，析构由 hicc `Drop` 接管。

## 构建方法

### C++ 独立验证（任选其一）

```bash
cd cpp
./standalone.sh          # 直接编译运行
make run                 # 或 Makefile
cmake -B build && cmake --build build   # 或 CMake
```

### Rust 编译与测试

```bash
cd rust_hicc
cargo test               # 运行 tests/smoke.rs 行为级断言
cargo run                # 运行 src/main.rs 演示
```

## 总结

1. **去 shim**：直接绑定 `class_basic_ns::Counter`，无 opaque 指针、无 `extern "C"` 桥接。
2. **make_unique 工厂**：每个公有构造派生一条 `hicc::make_unique` 工厂，替代 `*_new` shim。
3. **Drop 析构**：hicc 自动管理对象生命周期，替代 `*_delete` shim。
4. **ClassRef 引用返回**：`const std::string&` → `&string`（`hicc_std::string`）。
5. **零 unsafe 调用**：hicc 将 FFI 封装为安全 Rust API。
