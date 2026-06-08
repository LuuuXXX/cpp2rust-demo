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

```rust
hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <functional>
    #include <vector>
    #include <thread>
    #include <chrono>

    #include "std_function.h"
}

hicc::import_class! {
    #[cpp(class = "CallbackWrapper", destroy = "callback_wrapper_delete")]
    pub class CallbackWrapper {
        #[cpp(method = "int invoke(int value)")]
        fn invoke(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Processor", destroy = "processor_delete")]
    pub class Processor {
        #[cpp(method = "int process(int value)")]
        fn process(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "MultiCallback", destroy = "multi_callback_delete")]
    pub class MultiCallback {
        #[cpp(method = "void invoke_all(int value)")]
        fn invoke_all(&mut self, value: i32);
    }
}

hicc::import_class! {
    #[cpp(class = "AsyncProcessor", destroy = "async_processor_delete")]
    pub class AsyncProcessor {
        #[cpp(method = "bool is_cancelled() const")]
        fn is_cancelled(&self) -> bool;

        #[cpp(method = "void cancel()")]
        fn cancel(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "std_function"]

    class CallbackWrapper;
    class Processor;
    class MultiCallback;
    class AsyncProcessor;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "CallbackWrapper* callback_wrapper_new(int (*)(int))")]
    unsafe fn callback_wrapper_new(fn_: unsafe extern "C" fn(i32) -> i32) -> CallbackWrapper;

    #[cpp(func = "CallbackWrapper* callback_wrapper_new_double()")]
    fn callback_wrapper_new_double() -> CallbackWrapper;

    #[cpp(func = "Processor* processor_new()")]
    fn processor_new() -> Processor;

    #[cpp(func = "MultiCallback* multi_callback_new()")]
    fn multi_callback_new() -> MultiCallback;

    #[cpp(func = "AsyncProcessor* async_processor_new()")]
    fn async_processor_new() -> AsyncProcessor;

    #[cpp(func = "void processor_set_double(Processor* p)")]
    unsafe fn processor_set_double(p: *mut Processor);

    #[cpp(func = "void multi_callback_add_double(MultiCallback* mc)")]
    unsafe fn multi_callback_add_double(mc: *mut MultiCallback);

    #[cpp(func = "void multi_callback_add_triple(MultiCallback* mc)")]
    unsafe fn multi_callback_add_triple(mc: *mut MultiCallback);
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
invoke(5) = 10 (doubles input)
invoke(7) = 14 (doubles input)

--- Processor Demo ---
process(10) = 20

--- MultiCallback Demo ---
Invoking all callbacks with 4:

--- AsyncProcessor Demo ---
is_cancelled = false
after cancel: is_cancelled = true

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
