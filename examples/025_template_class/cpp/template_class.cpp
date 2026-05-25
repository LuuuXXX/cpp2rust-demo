#include "template_class.h"
#include <iostream>
#include <stack>

IntStack* intstack_new(void) {
    return new IntStack();
}

void intstack_delete(IntStack* self) {
    delete self;
}

int intstack_size(IntStack* self) {
    return self->impl.size();
}

int intstack_empty(IntStack* self) {
    return self->impl.empty() ? 1 : 0;
}

void intstack_push(IntStack* self, int value) {
    self->impl.push(value);
}

int intstack_top(IntStack* self) {
    return self->impl.top();
}

void intstack_pop(IntStack* self) {
    self->impl.pop();
}

DoubleStack* doublestack_new(void) {
    return new DoubleStack();
}

void doublestack_delete(DoubleStack* self) {
    delete self;
}

int doublestack_size(DoubleStack* self) {
    return self->impl.size();
}

int doublestack_empty(DoubleStack* self) {
    return self->impl.empty() ? 1 : 0;
}

void doublestack_push(DoubleStack* self, double value) {
    self->impl.push(value);
}

double doublestack_top(DoubleStack* self) {
    return self->impl.top();
}

void doublestack_pop(DoubleStack* self) {
    self->impl.pop();
}
