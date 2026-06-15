#include "noexcept_basic.h"
#include <iostream>
#include <stdexcept>
#include <utility>

NoexceptMover::NoexceptMover(int value) : value_(value) {}
NoexceptMover::~NoexceptMover() {}
NoexceptMover::NoexceptMover(NoexceptMover&& other) noexcept : value_(other.value_) {
    other.value_ = 0;
}
NoexceptMover& NoexceptMover::operator=(NoexceptMover&& other) noexcept {
    if (this != &other) {
        value_ = other.value_;
        other.value_ = 0;
    }
    return *this;
}
int NoexceptMover::get_value() const {
    return value_;
}
