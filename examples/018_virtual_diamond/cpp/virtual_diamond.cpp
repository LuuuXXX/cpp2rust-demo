#include "virtual_diamond.h"
#include <iostream>

A::A(int v) : a_value(v) {}

A::~A() {}

int A::getAValue() const {
    return a_value;
}

B::B(int a, int b) : A(a), b_value(b) {}

B::~B() {}

int B::getBValue() const {
    return b_value;
}

C::C(int a, int c) : A(a), c_value(c) {}

C::~C() {}

int C::getCValue() const {
    return c_value;
}

D::D(int a, int b, int c, int d) : A(a), B(a, b), C(a, c), d_value(d) {}

D::~D() {}

int D::getDValue() const {
    return d_value;
}

void D::compute() const {
    std::cout << "D::compute: a=" << a_value << " b=" << b_value
              << " c=" << c_value << " d=" << d_value << std::endl;
    std::cout << "Sum: " << (a_value + b_value + c_value + d_value) << std::endl;
}
