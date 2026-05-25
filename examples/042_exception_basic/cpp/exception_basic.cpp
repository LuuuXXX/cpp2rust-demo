#include "exception_basic.h"
#include <iostream>
#include <stdexcept>
#include <cstring>

// ExceptionInfo implementation
ExceptionInfo::ExceptionInfo() : code(EXCEPTION_NONE) {
    message[0] = '\0';
}
void ExceptionInfo::clear() {
    code = EXCEPTION_NONE;
    message[0] = '\0';
}
void ExceptionInfo::set(int c, const char* msg) {
    code = c;
    strncpy(message, msg, 255);
    message[255] = '\0';
}

// CalculatorImpl implementation
CalculatorImpl::CalculatorImpl() {}
CalculatorImpl::~CalculatorImpl() {}
void CalculatorImpl::clear_exception() {
    last_exception.clear();
}
int CalculatorImpl::get_exception() {
    return last_exception.code;
}
int CalculatorImpl::divide(int a, int b) {
    if (b == 0) {
        last_exception.set(EXCEPTION_RUNTIME_ERROR, "Division by zero");
        throw std::runtime_error("Division by zero");
    }
    return a / b;
}
int CalculatorImpl::safe_get(int* arr, int size, int index) {
    if (index < 0 || index >= size) {
        last_exception.set(EXCEPTION_OUT_OF_RANGE, "Index out of range");
        throw std::out_of_range("Index out of range");
    }
    return arr[index];
}
int CalculatorImpl::string_to_int(const char* str) {
    if (!str || *str == '\0') {
        last_exception.set(EXCEPTION_INVALID_ARGUMENT, "Empty string");
        throw std::invalid_argument("Empty string");
    }
    char* end;
    int result = std::strtol(str, &end, 10);
    if (*end != '\0') {
        last_exception.set(EXCEPTION_INVALID_ARGUMENT, "Invalid number format");
        throw std::invalid_argument("Invalid number format");
    }
    return result;
}

// Calculator implementation
Calculator::Calculator() : impl(new CalculatorImpl()) {}
Calculator::~Calculator() { delete impl; }

// Calculator C API implementation
struct Calculator* calculator_new(void) {
    return new Calculator();
}

void calculator_delete(struct Calculator* self) {
    delete self;
}

int calculator_get_exception(const struct Calculator* self) {
    if (self) return self->impl->get_exception();
    return EXCEPTION_RUNTIME_ERROR;
}

void calculator_clear_exception(struct Calculator* self) {
    if (self) self->impl->clear_exception();
}

int calculator_divide(struct Calculator* self, int a, int b) {
    if (self) return self->impl->divide(a, b);
    return 0;
}

int calculator_safe_get(struct Calculator* self, int* arr, int size, int index) {
    if (self) return self->impl->safe_get(arr, size, index);
    return 0;
}

int string_to_int(struct Calculator* self, const char* str) {
    if (self) return self->impl->string_to_int(str);
    return 0;
}

int has_exception(const struct Calculator* self) {
    if (self) return self->impl->get_exception() != EXCEPTION_NONE;
    return 1;
}
