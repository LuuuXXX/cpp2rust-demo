#include "virtual_diamond.h"

namespace virtual_diamond_ns {

A::A(int v) : a_value_(v) {}
A::~A() = default;
int A::a_value() const { return a_value_; }

B::B(int a, int b) : A(a), b_value_(b) {}
B::~B() = default;
int B::b_value() const { return b_value_; }

C::C(int a, int c) : A(a), c_value_(c) {}
C::~C() = default;
int C::c_value() const { return c_value_; }

// 虚继承：D 直接初始化唯一的 A 子对象
D::D(int a, int b, int c, int d)
    : A(a), B(a, b), C(a, c), d_value_(d) {}
D::~D() = default;
int D::d_value() const { return d_value_; }
int D::compute() const { return a_value_ + b_value_ + c_value_ + d_value_; }

int virtual_diamond_anchor() { return 0; }

} // namespace virtual_diamond_ns
