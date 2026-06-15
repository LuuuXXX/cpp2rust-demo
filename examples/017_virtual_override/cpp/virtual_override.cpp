#include "virtual_override.h"
#include <string>

Base::Base(const char* n) : name(n) {}

Base::~Base() {}

double Base::area() const {
    return 0.0;
}

const char* Base::getName() const {
    return name.c_str();
}

Derived::Derived(double v) : Base("Derived"), value(v) {}

Derived::~Derived() {}

double Derived::area() const {
    return value * value;
}

double Derived::getValue() const {
    return value;
}
