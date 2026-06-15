#include "inheritance_multiple.h"
#include <iostream>

Base1::Base1(int v) : value1(v) {}

Base1::~Base1() {}

int Base1::getValue1() const {
    return value1;
}

Base2::Base2(int v) : value2(v) {}

Base2::~Base2() {}

int Base2::getValue2() const {
    return value2;
}

Derived::Derived(int v1, int v2, int dv) : Base1(v1), Base2(v2), derived_value(dv) {}

Derived::~Derived() {}

int Derived::getDerivedValue() const {
    return derived_value;
}

void Derived::compute() const {
    std::cout << "Computing: " << value1 << " + " << value2 << " + " << derived_value
              << " = " << (value1 + value2 + derived_value) << std::endl;
}
