#include "operator_overload.h"

Number::Number(int v) : value(v) {}
Number::~Number() {}
int Number::getValue() const { return value; }
Number Number::operator+(const Number& other) const { return Number(value + other.value); }
Number Number::operator-(const Number& other) const { return Number(value - other.value); }
Number Number::operator*(const Number& other) const { return Number(value * other.value); }
Number Number::operator/(const Number& other) const { return Number(value / other.value); }
int Number::compare(const Number& other) const { return value - other.value; }
Number Number::operator-() const { return Number(-value); }
Number& Number::operator++() { ++value; return *this; }
Number& Number::operator--() { --value; return *this; }
Number& Number::operator+=(const Number& other) { value += other.value; return *this; }
Number& Number::operator-=(const Number& other) { value -= other.value; return *this; }
