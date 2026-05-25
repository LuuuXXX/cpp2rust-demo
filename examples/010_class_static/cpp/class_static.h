#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct Counter;

// 实例方法
struct Counter* counter_new(void);
void counter_delete(struct Counter* self);
int counter_getValue(struct Counter* self);
void counter_increment(struct Counter* self);

// 静态方法
int counter_getInstanceCount(void);
void counter_resetInstanceCount(void);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
class Counter {
    int value;
    static int instance_count;
public:
    Counter();
    ~Counter();
    int getValue() const;
    void increment();
    static int getInstanceCount();
    static void resetInstanceCount();
};

#endif
