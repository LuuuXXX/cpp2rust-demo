#include "virtual_override.h"

namespace virtual_override_ns {

Base::Base() = default;
Base::~Base() = default;
double Base::area() const { return 0.0; }

Derived::Derived(double v) : value_(v) {}
Derived::~Derived() = default;
double Derived::area() const { return value_ * value_; }
double Derived::value() const { return value_; }

int virtual_override_anchor() { return 0; }

} // namespace virtual_override_ns
