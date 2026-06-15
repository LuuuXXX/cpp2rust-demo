#include "direct_class_basic.h"

Counter::Counter() : value(0) {}
Counter::~Counter() {}
int Counter::get() const { return value; }
void Counter::increment() { value++; }
void Counter::decrement() { value--; }
