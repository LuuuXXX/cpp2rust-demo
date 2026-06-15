#include "explicit_ctor.h"

Widget::Widget(int v) : value(v) {}
Widget::Widget(double v) : value(static_cast<int>(v)) {}
Widget::~Widget() {}
int Widget::getValue() const { return value; }
