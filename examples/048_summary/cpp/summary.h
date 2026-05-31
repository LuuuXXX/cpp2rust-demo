#pragma once
#include <cstdint>

#ifdef __cplusplus
extern "C" {
#endif

struct Counter;

struct Counter* counter_new();
void counter_delete(struct Counter* self);
int safe_add(int a, int b);
int get_max_size();

#ifdef __cplusplus
}
#endif

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
