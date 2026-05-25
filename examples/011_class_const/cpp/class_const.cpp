#include "class_const.h"
#include <iostream>
#include <vector>

// Calculator class implementations
Calculator::Calculator() : value(0) {}

Calculator::~Calculator() {}

int Calculator::getValue() const {
    return value;
}

int Calculator::getHistoryCount() const {
    return static_cast<int>(history.size());
}

void Calculator::add(int v) {
    history.push_back(v);
    value += v;
}

void Calculator::subtract(int v) {
    history.push_back(-v);
    value -= v;
}

void Calculator::clear() {
    history.clear();
    value = 0;
}

// FFI wrapper functions
struct Calculator* calculator_new(void) {
    return new Calculator();
}

void calculator_delete(struct Calculator* self) {
    delete self;
}

int calculator_getValue(const struct Calculator* self) {
    return self->getValue();
}

int calculator_getHistoryCount(const struct Calculator* self) {
    return self->getHistoryCount();
}

void calculator_add(struct Calculator* self, int value) {
    self->add(value);
}

void calculator_subtract(struct Calculator* self, int value) {
    self->subtract(value);
}

void calculator_clear(struct Calculator* self) {
    self->clear();
}
