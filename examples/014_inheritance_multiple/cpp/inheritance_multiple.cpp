#include "inheritance_multiple.h"
#include <iostream>

// Base1 class implementations
Base1::Base1(int v) : value1(v) {}

Base1::~Base1() {}

int Base1::getValue1() const {
    return value1;
}

// Base2 class implementations
Base2::Base2(int v) : value2(v) {}

Base2::~Base2() {}

int Base2::getValue2() const {
    return value2;
}

// Derived class implementations
Derived::Derived(int v1, int v2, int dv) : Base1(v1), Base2(v2), derived_value(dv) {}

Derived::~Derived() {}

int Derived::getDerivedValue() const {
    return derived_value;
}

void Derived::compute() const {
    std::cout << "Computing: " << value1 << " + " << value2 << " + " << derived_value
              << " = " << (value1 + value2 + derived_value) << std::endl;
}

// FFI wrapper functions
struct Derived* derived_new(int v1, int v2, int dv) {
    return new Derived(v1, v2, dv);
}

void derived_delete(struct Derived* self) {
    delete self;
}

int derived_getValue1(struct Derived* self) {
    return self->getValue1();
}

int derived_getValue2(struct Derived* self) {
    return self->getValue2();
}

int derived_getDerivedValue(struct Derived* self) {
    return self->getDerivedValue();
}

void derived_compute(struct Derived* self) {
    self->compute();
}
