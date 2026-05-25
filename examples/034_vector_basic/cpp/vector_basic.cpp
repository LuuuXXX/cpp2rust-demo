#include "vector_basic.h"
#include <iostream>
#include <vector>
#include <string>
#include <cstring>

// IntVectorImpl class implementation
IntVectorImpl::IntVectorImpl() : data() {
}

IntVectorImpl::~IntVectorImpl() {
    data.clear();
}

// StringVectorImpl class implementation
StringVectorImpl::StringVectorImpl() : data() {
}

StringVectorImpl::~StringVectorImpl() {
    data.clear();
}

// IntVector struct implementation
IntVector::IntVector() : impl(new IntVectorImpl()) {
}

IntVector::~IntVector() {
    delete impl;
    impl = nullptr;
}

// StringVector struct implementation
StringVector::StringVector() : impl(new StringVectorImpl()) {
}

StringVector::~StringVector() {
    delete impl;
    impl = nullptr;
}

// FFI wrapper functions
struct IntVector* int_vector_new(void) {
    return new IntVector();
}

void int_vector_delete(struct IntVector* self) {
    delete self;
}

size_t int_vector_size(const struct IntVector* self) {
    return self->impl->data.size();
}

size_t int_vector_capacity(const struct IntVector* self) {
    return self->impl->data.capacity();
}

int int_vector_empty(const struct IntVector* self) {
    return self->impl->data.empty() ? 1 : 0;
}

void int_vector_push_back(struct IntVector* self, int value) {
    self->impl->data.push_back(value);
}

void int_vector_pop_back(struct IntVector* self) {
    if (!self->impl->data.empty()) {
        self->impl->data.pop_back();
    }
}

int int_vector_get(const struct IntVector* self, size_t index) {
    if (index < self->impl->data.size()) {
        return self->impl->data[index];
    }
    return 0;
}

void int_vector_set(struct IntVector* self, size_t index, int value) {
    if (index < self->impl->data.size()) {
        self->impl->data[index] = value;
    }
}

void int_vector_clear(struct IntVector* self) {
    self->impl->data.clear();
}

int* int_vector_data(struct IntVector* self) {
    return self->impl->data.data();
}

// StringVector C API implementation
struct StringVector* string_vector_new(void) {
    return new StringVector();
}

void string_vector_delete(struct StringVector* self) {
    delete self;
}

size_t string_vector_size(const struct StringVector* self) {
    return self->impl->data.size();
}

size_t string_vector_capacity(const struct StringVector* self) {
    return self->impl->data.capacity();
}

void string_vector_push_back(struct StringVector* self, const char* value) {
    if (value) {
        self->impl->data.push_back(std::string(value));
    }
}

const char* string_vector_get(const struct StringVector* self, size_t index) {
    static thread_local std::string temp;
    if (index < self->impl->data.size()) {
        temp = self->impl->data[index];
        return temp.c_str();
    }
    temp = "";
    return temp.c_str();
}

void string_vector_clear(struct StringVector* self) {
    self->impl->data.clear();
}
