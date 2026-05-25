#pragma once

#ifdef __cplusplus
extern "C" {
#endif

class Counter;

struct Counter* counter_new(void);
void counter_delete(struct Counter* self);
int counter_get(struct Counter* self);
void counter_increment(struct Counter* self);
void counter_decrement(struct Counter* self);

#ifdef __cplusplus
}
#endif

#ifdef __cplusplus
class Counter {
    int value = 0;
public:
    Counter();
    ~Counter();
    int get() const;
    void increment();
    void decrement();
};
#endif
