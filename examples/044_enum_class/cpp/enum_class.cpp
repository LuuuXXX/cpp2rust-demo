#include "enum_class.h"
#include <iostream>

namespace example {

OperationResult::OperationResult() : error_(ErrorCode::None), state_(State::Idle), flags_(Flags::None) {}
OperationResult::~OperationResult() {}
void OperationResult::set_error(int code) {
    error_ = static_cast<ErrorCode>(code);
}
int OperationResult::get_error() const {
    return static_cast<int>(error_);
}
void OperationResult::set_state(unsigned char s) {
    state_ = static_cast<State>(s);
}
unsigned char OperationResult::get_state() const {
    return static_cast<unsigned char>(state_);
}
void OperationResult::set_flags(unsigned int f) {
    flags_ = static_cast<Flags>(f);
}
unsigned int OperationResult::get_flags() const {
    return static_cast<unsigned int>(flags_);
}

}  // namespace example
