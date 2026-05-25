#include "explicit_ctor.h"
#include <iostream>

struct Widget* widget_new(int value) {
    return new Widget(value);
}

struct Widget* widget_fromInt(int value) {
    return new Widget(value);
}

struct Widget* widget_fromDouble(double value) {
    return new Widget(value);
}

void widget_delete(struct Widget* self) {
    delete self;
}

int widget_getValue(struct Widget* self) {
    return self->getValue();
}

// Widget class implementation
Widget::Widget(int v) : value(v) {}
Widget::Widget(double v) : value(static_cast<int>(v)) {}
Widget::~Widget() {}
int Widget::getValue() const { return value; }
