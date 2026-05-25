#include "virtual_diamond.h"
#include <iostream>

// A class implementations
A::A(int v) : a_value(v) {}

A::~A() {}

int A::getAValue() const {
    return a_value;
}

// B class implementations
B::B(int a, int b) : A(a), b_value(b) {}

B::~B() {}

int B::getBValue() const {
    return b_value;
}

// C class implementations
C::C(int a, int c) : A(a), c_value(c) {}

C::~C() {}

int C::getCValue() const {
    return c_value;
}

// D class implementations
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

// FFI wrapper functions
struct D* d_new(int a, int b, int c, int d) {
    return new D(a, b, c, d);
}

void d_delete(struct D* self) {
    delete self;
}

int d_getAValue(struct D* self) {
    std::cout << "Getting A value (virtual base - single instance)" << std::endl;
    return self->getAValue();
}

int d_getBValue(struct D* self) {
    return self->getBValue();
}

int d_getCValue(struct D* self) {
    return self->getCValue();
}

int d_getDValue(struct D* self) {
    return self->getDValue();
}

void d_compute(struct D* self) {
    self->compute();
}
