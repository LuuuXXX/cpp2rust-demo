#include "class_const.h"
#include <iostream>
#include <vector>

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
