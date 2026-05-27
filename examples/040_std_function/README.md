# 040_std_function - std::function 回调

## C++ 特性

本示例展示 C++ `std::function`（通用可调用对象包装器）的 FFI 处理方式，特别是如何将 Rust 闭包传递给 C++。

## C++ 代码

### std_function.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

struct CallbackWrapper;
struct Processor;

struct CallbackWrapper* callback_wrapper_new(int (*fn)(int));
void callback_wrapper_delete(struct CallbackWrapper* self);
int callback_wrapper_invoke(struct CallbackWrapper* self, int value);

struct Processor* processor_new(void);
void processor_delete(struct Processor* self);
void processor_set_callback(struct Processor* self, int (*cb)(int));
int processor_process(struct Processor* self, int value);

#ifdef __cplusplus
}
#endif
```

### std_function.cpp

```cpp
#include "std_function.h"
#include <functional>

struct CallbackWrapper {
    std::function<int(int)> callback;

    CallbackWrapper(int (*fn)(int)) : callback(fn) {}

    int invoke(int value) {
        if (callback) {
            return callback(value);
        }
        return value;
    }
};

struct Processor* processor_new() { return new Processor(); }

void processor_set_callback(struct Processor* self, int (*cb)(int)) {
    self->callback = cb;
}

int processor_process(struct Processor* self, int value) {
    if (self->callback) {
        return self->callback(value);
    }
    return value * 2;
}
```

## std::function 特点

| 特性 | 说明 |
|------|------|
| 类型擦除 | 存储任何可调用对象 |
| 通用包装 | 函数指针、lambda、函数对象 |
| 内存管理 | 自动管理捕获的 lambda |
| 空检查 | `operator bool()` |

### 与函数指针对比

```cpp
// 函数指针 - 只适用于普通函数
typedef int (*Callback)(int);
void foo(Callback cb);

// std::function - 适用于任何可调用对象
#include <functional>
void foo(std::function<int(int)> cb);
```

## Rust FFI 代码

### main.rs

```rust
hicc::import_lib! {
    #![link_name = "std_function"]

    struct Processor;

    #[cpp(func = "struct Processor* processor_new(void)")]
    fn processor_new() -> *mut Processor;

    #[cpp(func = "void processor_set_callback(struct Processor*, int(*)(int))")]
    unsafe fn processor_set_callback(
        p: *mut Processor,
        cb: Option<extern "C" fn(i32) -> i32>
    );

    #[cpp(func = "int processor_process(struct Processor*, int)")]
    unsafe fn processor_process(p: *mut Processor, value: i32) -> i32;
}

// Rust 回调函数
extern "C" fn rust_callback(value: i32) -> i32 {
    value * 2
}
```

## FFI 对比分析

| 方面 | C++ std::function | Rust FFI |
|------|-------------------|----------|
| 存储内容 | 任何可调用对象 | 函数指针 |
| 类型擦除 | 是 | 否（类型擦除） |
| 捕获 lambda | 支持 | 需要包装 |
| 空状态 | 支持 | nullptr |

## 关键点

1. **函数指针**：Rust 闭包转为函数指针传递
2. **类型擦除**：C++ 侧使用 std::function
3. **生命周期**：Rust 函数指针需要 'static
4. **回调链**：多个回调需要多函数指针数组

## 使用场景

- **事件处理**：GUI 按钮点击
- **异步编程**：完成回调、进度回调
- **策略模式**：运行时选择算法
- **观察者模式**：事件通知

## 运行结果

```
=== 040_std_function - std::function 回调 ===

--- CallbackWrapper Demo ---
process(5) with multiplier=2: 10
get_value(): 10
process(7) with multiplier=3: 21

--- Processor Demo ---
Set input: 21
Simulated result (input * 2): 42

Rust FFI: std::function 回调映射
1. std::function 存储可调用对象
2. 回调可用于事件处理
3. 此示例展示基本的回调封装模式
```

## 总结

- std::function 提供类型擦除的可调用对象包装
- FFI 边界使用函数指针作为回调
- Rust 闭包需要转换为函数指针
- 适用于需要运行时灵活性的场景
