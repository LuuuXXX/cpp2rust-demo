# 041_functional_bind - std::bind 绑定

## C++ 特性

本示例展示如何使用 std::bind 创建部分应用的函数对象，以及如何通过 FFI 传递绑定了参数的函数。

## C++ 代码

### functional_bind.h

```cpp
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// Adder - 绑定基础值
struct Adder* adder_new(int base_value);
void adder_delete(struct Adder* self);
int adder_add(struct Adder* self, int value);

// 顶层函数绑定
int add_five(int a);
int add_ten(int a);

// Multiplier - 绑定乘数
struct Multiplier* multiplier_new(int factor);
void multiplier_delete(struct Multiplier* self);
int multiply(struct Multiplier* self, int value);

// StringProcessor - 成员函数绑定
struct StringProcessor* string_processor_new(void);
void string_processor_delete(struct StringProcessor* self);
void string_processor_set_target(struct StringProcessor* self, const char* target);
int string_processor_count_char(struct StringProcessor* self, char ch);

#ifdef __cplusplus
}
#endif
```

### functional_bind.cpp

```cpp
#include "functional_bind.h"
#include <functional>
#include <string>

// Adder 实现
struct Adder {
    int base_value;
    Adder(int base) : base_value(base) {}

    int add(int value) {
        return base_value + value;  // base_value 已绑定
    }
};

// Multiplier 实现
struct Multiplier {
    int factor;
    Multiplier(int f) : factor(f) {}

    int multiply(int value) {
        return value * factor;  // factor 已绑定
    }
};

// std::bind 的实际使用
// 在 C++ 中：
// auto add_five = std::bind(add, std::placeholders::_1, 5);
// auto multiply_by_seven = std::bind(multiply, 7, std::placeholders::_1);
```

## Rust FFI 代码

### main.rs

```rust
hicc::import_lib! {
    #![link_name = "functional_bind"]

    struct Adder;
    #[cpp(func = "struct Adder* adder_new(int)")]
    fn adder_new(base_value: i32) -> *mut Adder;

    // ... 其他声明
}

// 使用绑定了参数的函数
let adder = adder_new(100);
adder_add(adder, 50);  // 相当于 100 + 50 = 150
```

## 构建方法

### C++ 编译

```bash
cd cpp
g++ -c functional_bind.cpp -o functional_bind.o
g++ -shared -fPIC functional_bind.cpp -o libfunctional_bind.so
```

### Rust 编译

```bash
cd rust_hicc
cargo build
```

## FFI 对比分析

| 方面 | C++ | Rust |
|------|-----|------|
| 绑定基础值 | `std::bind(&Adder::add, obj, _1)` | 通过 opaque pointer 封装 |
| 绑定乘数 | `std::bind(multiply, 7, _1)` | 通过结构体存储 factor |
| 成员函数绑定 | `std::bind(&Class::method, obj, _1)` | 间接调用 |
| 占位符 | `std::placeholders::_1` | 通过参数传递实现 |

## 总结

1. `std::bind` 创建部分应用的函数对象
2. 绑定的值成为函数对象状态的一部分
3. 通过 opaque pointer 在 FFI 间传递绑定了参数的函数
4. `_1, _2` 等占位符表示未绑定的参数位置
5. 可以绑定普通函数、成员函数、Lambda 表达式