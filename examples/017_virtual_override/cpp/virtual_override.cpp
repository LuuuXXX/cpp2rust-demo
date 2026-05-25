#include "virtual_override.h"
#include <iostream>
#include <cstring>
#include <string>

// Base class implementations
Base::Base(const char* n) : name(n) {}

Base::~Base() {}

double Base::area() const {
    return 0.0;
}

const char* Base::getName() const {
    return name.c_str();
}

// Derived class implementations
Derived::Derived(double v) : Base("Derived"), value(v) {}

Derived::~Derived() {}

double Derived::area() const {
    return value * value;  // area = value^2 for demonstration
}

double Derived::getValue() const {
    return value;
}

// FFI wrapper functions
struct Base* base_create(int type) {
    if (type == 0) {
        std::cout << "Creating Base" << std::endl;
        return new Base("Base");
    } else {
        std::cout << "Creating Derived (as Base*)" << std::endl;
        return new Derived(42.0);
    }
}

void base_delete(struct Base* self) {
    delete self;
}

double base_area(struct Base* self) {
    return self->area();
}

const char* base_getName(struct Base* self) {
    return self->getName();
}

struct Derived* derived_new(double value) {
    return new Derived(value);
}

void derived_delete(struct Derived* self) {
    delete self;
}

double derived_getValue(struct Derived* self) {
    return self->getValue();
}
