#pragma once

#include <stack>

namespace template_class_ns {

// 类模板：一个简单的泛型栈。模板是「蓝图」，须按具体类型实例化（Stack<int> …）
// 才成为可链接的具体类型。
template <typename T>
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

// 显式实例化为具体类：每个具体类型暴露一个 idiomatic 命名空间类，内部复用 Stack<T>。
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

// 锚点：本单元可链接的非模板符号。
int template_class_anchor();

} // namespace template_class_ns
