#include "class_static.h"
#include <iostream>

int Counter::instance_count = 0;

Counter::Counter() : value(0) {
    instance_count++;
}

Counter::~Counter() {
    instance_count--;
}

int Counter::getValue() const {
    return value;
}

void Counter::increment() {
    value++;
}

int Counter::getInstanceCount() {
    return instance_count;
}

void Counter::resetInstanceCount() {
    instance_count = 0;
}
