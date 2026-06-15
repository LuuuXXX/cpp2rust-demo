#pragma once

// Direct 模式示例：纯 C++ 类，无 extern "C" shim 访问器。
// 工具通过 hicc::make_unique<T> + #[cpp(method = "...")] 直接绑定方法，
// 无需 counter_new / counter_get 等 C 包装函数。

class Counter {
    int value = 0;
public:
    Counter();
    ~Counter();
    int get() const;
    void increment();
    void decrement();
};
