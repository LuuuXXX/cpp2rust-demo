#include "class_basic.h"
#include <iostream>

Counter::Counter() : value(0) {}
Counter::~Counter() {}
int Counter::get() const { return value; }
void Counter::increment() { value++; }
void Counter::decrement() { value--; }
