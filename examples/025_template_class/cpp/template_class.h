#pragma once

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
