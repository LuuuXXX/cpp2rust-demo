#include "operator_overload.h"
#include <iostream>

struct Number* number_new(int value) {
    return new Number(value);
}

void number_delete(struct Number* self) {
    delete self;
}

int number_getValue(struct Number* self) {
    return self->getValue();
}

struct Number* number_add(struct Number* self, struct Number* other) {
    return new Number(self->operator+(*other));
}

struct Number* number_sub(struct Number* self, struct Number* other) {
    return new Number(self->operator-(*other));
}

struct Number* number_mul(struct Number* self, struct Number* other) {
    return new Number(self->operator*(*other));
}

struct Number* number_div(struct Number* self, struct Number* other) {
    return new Number(self->operator/(*other));
}

int number_compare(struct Number* self, struct Number* other) {
    return self->compare(*other);
}

struct Number* number_negate(struct Number* self) {
    return new Number(self->operator-());
}

struct Number* number_increment(struct Number* self) {
    return &self->operator++();
}

struct Number* number_decrement(struct Number* self) {
    return &self->operator--();
}

void number_add_assign(struct Number* self, struct Number* other) {
    self->operator+=(*other);
}

void number_sub_assign(struct Number* self, struct Number* other) {
    self->operator-=(*other);
}

// Number class implementation
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
