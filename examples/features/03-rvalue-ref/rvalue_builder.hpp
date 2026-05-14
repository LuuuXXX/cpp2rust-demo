#pragma once
// Demonstrates how cpp2rust-demo maps C++ ref-qualifier methods:
//
//  &   (lvalue) const method  → fn foo(&self)
//  &   (lvalue) mutable method → fn foo(&mut self)
//  &&  (rvalue) method         → fn foo(self)   (consumes the object)
//
// This example uses a simple `Builder` class that exposes all three
// ref-qualifier variants.

class Builder {
public:
    Builder(int base);

    /// Const lvalue method — maps to `fn get(&self) -> i32`.
    int get() const;

    /// Mutable lvalue method — maps to `fn set(&mut self, v: i32)`.
    void set(int v);

    /// Rvalue-reference ("move") method — maps to `fn build(self) -> i32`.
    /// Calling this method consumes the builder.
    int build() &&;

    /// Plain (non-const, non-rvalue) method for comparison.
    void reset();
};
