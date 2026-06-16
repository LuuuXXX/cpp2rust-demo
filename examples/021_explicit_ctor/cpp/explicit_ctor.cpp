#include "explicit_ctor.h"

namespace explicit_ctor_ns {

Widget::Widget(int v) : value_(v) {}
Widget::Widget(double v) : value_(static_cast<int>(v)) {}
Widget::~Widget() = default;

int Widget::getValue() const { return value_; }

int explicit_ctor_anchor() { return 0; }

} // namespace explicit_ctor_ns
