#include "enum_class.h"

namespace enum_class_ns {

OperationResult::OperationResult()
    : error_(ErrorCode::None), state_(State::Idle), flags_(Flags::None) {}

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

unsigned int combine_flags(unsigned int f1, unsigned int f2) {
    return f1 | f2;
}

int has_flag(unsigned int flags, unsigned int flag) {
    return (flags & flag) != 0 ? 1 : 0;
}

int enum_class_anchor() { return 0; }

} // namespace enum_class_ns
