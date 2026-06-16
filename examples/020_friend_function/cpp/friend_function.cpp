#include "friend_function.h"

namespace friend_function_ns {

MyClass::MyClass(int v) : value_(v) {}
MyClass::~MyClass() = default;

int MyClass::getValue() const { return value_; }
void MyClass::setValue(int v) { value_ = v; }

int friend_function_anchor() { return 0; }

} // namespace friend_function_ns
