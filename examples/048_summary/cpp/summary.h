#pragma once
#include <cstdint>

class Counter {
    int value = 0;
public:
    Counter() = default;
    ~Counter() = default;
    int get() const { return value; }
    void increment() { value++; }
    void decrement() { value--; }
};

Counter* counter_new();
void counter_delete(Counter* self);
int safe_add(int a, int b) noexcept;
int get_max_size();
