#include "inheritance_multiple.h"

namespace inheritance_multiple_ns {

Base1::Base1(int v) : value1_(v) {}
Base1::~Base1() = default;
int Base1::value1() const { return value1_; }

Base2::Base2(int v) : value2_(v) {}
Base2::~Base2() = default;
int Base2::value2() const { return value2_; }

Derived::Derived(int v1, int v2, int dv)
    : Base1(v1), Base2(v2), derived_value_(dv) {}
Derived::~Derived() = default;
int Derived::derived_value() const { return derived_value_; }
int Derived::compute() const { return value1_ + value2_ + derived_value_; }

int inheritance_multiple_anchor() { return 0; }

} // namespace inheritance_multiple_ns
