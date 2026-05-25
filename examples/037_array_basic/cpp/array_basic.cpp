#include "array_basic.h"
#include <iostream>
#include <array>
#include <string>
#include <cstring>

// IntArray5Impl class implementation
IntArray5Impl::IntArray5Impl() : data() {
}

IntArray5Impl::IntArray5Impl(const int* values) : data() {
    if (values) {
        for (size_t i = 0; i < 5; ++i) {
            data[i] = values[i];
        }
    }
}

IntArray5Impl::~IntArray5Impl() {
}

// DoubleArray3Impl class implementation
DoubleArray3Impl::DoubleArray3Impl() : data() {
}

DoubleArray3Impl::DoubleArray3Impl(const double* values) : data() {
    if (values) {
        for (size_t i = 0; i < 3; ++i) {
            data[i] = values[i];
        }
    }
}

DoubleArray3Impl::~DoubleArray3Impl() {
}

// StringArray4Impl class implementation
StringArray4Impl::StringArray4Impl() : data(), initialized{false, false, false, false} {
}

StringArray4Impl::~StringArray4Impl() {
}

// IntArray5 struct implementation
IntArray5::IntArray5() : impl(new IntArray5Impl()) {
}

IntArray5::IntArray5(const int* values) : impl(new IntArray5Impl(values)) {
}

IntArray5::~IntArray5() {
    delete impl;
    impl = nullptr;
}

// DoubleArray3 struct implementation
DoubleArray3::DoubleArray3() : impl(new DoubleArray3Impl()) {
}

DoubleArray3::DoubleArray3(const double* values) : impl(new DoubleArray3Impl(values)) {
}

DoubleArray3::~DoubleArray3() {
    delete impl;
    impl = nullptr;
}

// StringArray4 struct implementation
StringArray4::StringArray4() : impl(new StringArray4Impl()) {
}

StringArray4::~StringArray4() {
    delete impl;
    impl = nullptr;
}

// FFI wrapper functions
struct IntArray5* int_array5_new(void) {
    return new IntArray5();
}

struct IntArray5* int_array5_new_from(const int* values) {
    return new IntArray5(values);
}

void int_array5_delete(struct IntArray5* self) {
    delete self;
}

size_t int_array5_size(const struct IntArray5* self) {
    return self->impl->data.size();
}

int int_array5_empty(const struct IntArray5* self) {
    return self->impl->data.empty() ? 1 : 0;
}

int int_array5_get(const struct IntArray5* self, size_t index) {
    if (index < 5) {
        return self->impl->data[index];
    }
    return 0;
}

void int_array5_set(struct IntArray5* self, size_t index, int value) {
    if (index < 5) {
        self->impl->data[index] = value;
    }
}

int* int_array5_data(struct IntArray5* self) {
    return self->impl->data.data();
}

const int* int_array5_data_const(const struct IntArray5* self) {
    return self->impl->data.data();
}

int* int_array5_begin(struct IntArray5* self) {
    return self->impl->data.begin();
}

int* int_array5_end(struct IntArray5* self) {
    return self->impl->data.end();
}

int int_array5_at(const struct IntArray5* self, size_t index) {
    return self->impl->data.at(index);
}

void int_array5_swap(struct IntArray5* self, struct IntArray5* other) {
    self->impl->data.swap(other->impl->data);
}

// DoubleArray3 C API implementation
struct DoubleArray3* double_array3_new(void) {
    return new DoubleArray3();
}

struct DoubleArray3* double_array3_new_from(const double* values) {
    return new DoubleArray3(values);
}

void double_array3_delete(struct DoubleArray3* self) {
    delete self;
}

size_t double_array3_size(const struct DoubleArray3* self) {
    return self->impl->data.size();
}

double double_array3_get(const struct DoubleArray3* self, size_t index) {
    if (index < 3) {
        return self->impl->data[index];
    }
    return 0.0;
}

void double_array3_set(struct DoubleArray3* self, size_t index, double value) {
    if (index < 3) {
        self->impl->data[index] = value;
    }
}

double* double_array3_data(struct DoubleArray3* self) {
    return self->impl->data.data();
}

// StringArray4 C API implementation
struct StringArray4* string_array4_new(void) {
    return new StringArray4();
}

void string_array4_delete(struct StringArray4* self) {
    delete self;
}

size_t string_array4_size(const struct StringArray4* self) {
    return self->impl->data.size();
}

const char* string_array4_get(const struct StringArray4* self, size_t index) {
    static thread_local std::string temp;
    if (index < 4 && self->impl->initialized[index]) {
        temp = self->impl->data[index];
    } else {
        temp = "";
    }
    return temp.c_str();
}

void string_array4_set(struct StringArray4* self, size_t index, const char* value) {
    if (index < 4 && value) {
        self->impl->data[index] = std::string(value);
        self->impl->initialized[index] = true;
    }
}
