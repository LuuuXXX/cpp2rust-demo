// entry.cpp — conditional/01-template-no-alias
//
// STEP A (baseline): no alias → Stack<T> is skipped.
// Run cpp2rust-demo init with this file and observe the interface report.
//
// STEP B (unlocked): add the using aliases below, then re-run init.
// Uncomment the lines marked [UNLOCK] to enable extraction.

#include "stack.hpp"

// ── [UNLOCK] Add typedef/using aliases to unlock template extraction ──────
// using IntStack    = Stack<int>;
// using DoubleStack = Stack<double>;
// ─────────────────────────────────────────────────────────────────────────

// Stub implementations (only declarations matter for AST extraction)
template<typename T> Stack<T>::Stack()  : data_(nullptr), top_(0), cap_(0) {}
template<typename T> Stack<T>::~Stack() {}
template<typename T> void Stack<T>::push(T /*v*/) {}
template<typename T> T    Stack<T>::pop()         { return T{}; }
template<typename T> T    Stack<T>::top()  const  { return T{}; }
template<typename T> bool Stack<T>::empty() const { return top_ == 0; }
template<typename T> int  Stack<T>::size()  const { return top_; }
template<typename T> void Stack<T>::clear()       { top_ = 0; }
