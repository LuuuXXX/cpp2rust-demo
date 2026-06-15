#pragma once
#include <cstdint>

#ifdef __cplusplus
class Counter {
    int value = 0;
public:
    Counter() = default;
    ~Counter() = default;
    int get() const { return value; }
    void increment() { value++; }
    void decrement() { value--; }
};
#endif
