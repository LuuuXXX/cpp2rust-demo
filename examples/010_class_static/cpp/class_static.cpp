#include "class_static.h"
#include <iostream>

int Counter::instance_count = 0;

// Counter class implementations
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

// FFI wrapper functions
struct Counter* counter_new(void) {
    return new Counter();
}

void counter_delete(struct Counter* self) {
    delete self;
}

int counter_getValue(struct Counter* self) {
    return self->getValue();
}

void counter_increment(struct Counter* self) {
    self->increment();
}

int counter_getInstanceCount(void) {
    return Counter::getInstanceCount();
}

void counter_resetInstanceCount(void) {
    Counter::resetInstanceCount();
}
