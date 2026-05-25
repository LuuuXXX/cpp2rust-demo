#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct IntStack;
struct DoubleStack;

IntStack* intstack_new(void);
void intstack_delete(IntStack* self);

int intstack_size(IntStack* self);
int intstack_empty(IntStack* self);
void intstack_push(IntStack* self, int value);
int intstack_top(IntStack* self);
void intstack_pop(IntStack* self);

DoubleStack* doublestack_new(void);
void doublestack_delete(DoubleStack* self);

int doublestack_size(DoubleStack* self);
int doublestack_empty(DoubleStack* self);
void doublestack_push(DoubleStack* self, double value);
double doublestack_top(DoubleStack* self);
void doublestack_pop(DoubleStack* self);

#ifdef __cplusplus
}
#endif

#ifdef __cplusplus
#include <stack>
template<typename T>
class Stack {
public:
    std::stack<T> data;
    Stack() = default;
    int size() const { return static_cast<int>(data.size()); }
    bool empty() const { return data.empty(); }
    void push(T value) { data.push(value); }
    T top() const { return data.top(); }
    void pop() { data.pop(); }
};

class IntStack {
public:
    Stack<int> impl;
    IntStack() = default;
    int size() const { return impl.size(); }
    bool empty() const { return impl.empty(); }
    void push(int value) { impl.push(value); }
    int top() const { return impl.top(); }
    void pop() { impl.pop(); }
};

class DoubleStack {
public:
    Stack<double> impl;
    DoubleStack() = default;
    int size() const { return impl.size(); }
    bool empty() const { return impl.empty(); }
    void push(double value) { impl.push(value); }
    double top() const { return impl.top(); }
    void pop() { impl.pop(); }
};

#endif
