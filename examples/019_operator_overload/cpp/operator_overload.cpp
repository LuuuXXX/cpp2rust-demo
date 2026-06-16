#include "operator_overload.h"

namespace operator_overload_ns {

Number::Number(int v) : value_(v) {}
Number::~Number() = default;

int Number::value() const { return value_; }

Number Number::operator+(const Number& other) const { return Number(value_ + other.value_); }
Number Number::operator-(const Number& other) const { return Number(value_ - other.value_); }
Number Number::operator*(const Number& other) const { return Number(value_ * other.value_); }
Number Number::operator/(const Number& other) const { return Number(value_ / other.value_); }

int Number::compare(const Number& other) const {
    if (value_ < other.value_) return -1;
    if (value_ > other.value_) return 1;
    return 0;
}

Number Number::operator-() const { return Number(-value_); }
Number& Number::operator++() { ++value_; return *this; }
Number& Number::operator--() { --value_; return *this; }
Number& Number::operator+=(const Number& other) { value_ += other.value_; return *this; }
Number& Number::operator-=(const Number& other) { value_ -= other.value_; return *this; }

int operator_overload_anchor() { return 0; }

} // namespace operator_overload_ns
