#pragma once
// stack.hpp — conditional/01-template-no-alias
//
// A generic stack template WITHOUT any typedef/using alias.
// Demonstrates the ⚠️ conditional workflow:
//   • Without alias  → cpp2rust-demo skips the template (ToolConservative).
//   • After adding   `using IntStack = Stack<int>;`  in entry.cpp →
//     cpp2rust-demo automatically extracts the specialization.

template<typename T>
class Stack {
public:
    Stack();
    ~Stack();

    /// Push a value onto the top.
    void push(T value);

    /// Pop and return the top value (undefined behaviour if empty).
    T pop();

    /// Return the top value without removing it.
    T top() const;

    /// True if the stack contains no elements.
    bool empty() const;

    /// Number of elements currently on the stack.
    int size() const;

    /// Remove all elements.
    void clear();

private:
    T*  data_;
    int top_;
    int cap_;
};
