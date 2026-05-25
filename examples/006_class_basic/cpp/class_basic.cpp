#include "class_basic.h"
#include <iostream>

Counter::Counter() : value(0) {}
Counter::~Counter() {}
int Counter::get() const { return value; }
void Counter::increment() { value++; }
void Counter::decrement() { value--; }

struct Counter* counter_new(void) { return new Counter(); }
void counter_delete(struct Counter* self) { delete self; }
int counter_get(struct Counter* self) { return self->get(); }
void counter_increment(struct Counter* self) { self->increment(); }
void counter_decrement(struct Counter* self) { self->decrement(); }
