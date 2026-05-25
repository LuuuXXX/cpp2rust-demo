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

// FFI wrapper functions - directly return example::OperationResult*
example::OperationResult* operation_result_new(void) {
    return new example::OperationResult();
}

void operation_result_delete(example::OperationResult* p) {
    delete p;
}

void operation_result_set_error(example::OperationResult* p, int error_code) {
    if (p) p->set_error(error_code);
}

int operation_result_get_error(example::OperationResult* p) {
    if (p) return p->get_error();
    return 0;
}

void operation_result_set_state(example::OperationResult* p, unsigned char state) {
    if (p) p->set_state(state);
}

unsigned char operation_result_get_state(example::OperationResult* p) {
    if (p) return p->get_state();
    return 0;
}

void operation_result_set_flags(example::OperationResult* p, unsigned int flags) {
    if (p) p->set_flags(flags);
}

unsigned int operation_result_get_flags(example::OperationResult* p) {
    if (p) return p->get_flags();
    return 0;
}

unsigned int combine_flags(unsigned int f1, unsigned int f2) {
    return f1 | f2;
}

int has_flag(unsigned int flags, unsigned int flag) {
    return (flags & flag) == flag;
}
