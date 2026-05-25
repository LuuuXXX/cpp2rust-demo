#include "tuple_basic.h"
#include <iostream>
#include <tuple>
#include <string>
#include <cstring>

// Tuple2Impl class implementation
Tuple2Impl::Tuple2Impl(int first, const char* second) : data(first, second ? second : "") {
}

Tuple2Impl::~Tuple2Impl() {
}

// Tuple3Impl class implementation
Tuple3Impl::Tuple3Impl(int first, double second, const char* third)
    : data(first, second, third ? third : "") {
}

Tuple3Impl::~Tuple3Impl() {
}

// Tuple4Impl class implementation
Tuple4Impl::Tuple4Impl(int first, double second, const char* third, int fourth)
    : data(first, second, third ? third : "", fourth) {
}

Tuple4Impl::~Tuple4Impl() {
}

// Tuple2 struct implementation
Tuple2::Tuple2(int first, const char* second) : impl(new Tuple2Impl(first, second)) {
}

Tuple2::~Tuple2() {
    delete impl;
    impl = nullptr;
}

// Tuple3 struct implementation
Tuple3::Tuple3(int first, double second, const char* third)
    : impl(new Tuple3Impl(first, second, third)) {
}

Tuple3::~Tuple3() {
    delete impl;
    impl = nullptr;
}

// Tuple4 struct implementation
Tuple4::Tuple4(int first, double second, const char* third, int fourth)
    : impl(new Tuple4Impl(first, second, third, fourth)) {
}

Tuple4::~Tuple4() {
    delete impl;
    impl = nullptr;
}

// FFI wrapper functions
struct Tuple2* tuple2_new(int first, const char* second) {
    return new Tuple2(first, second);
}

void tuple2_delete(struct Tuple2* self) {
    delete self;
}

int tuple2_get_first(const struct Tuple2* self) {
    return std::get<0>(self->impl->data);
}

const char* tuple2_get_second(const struct Tuple2* self) {
    static thread_local std::string temp;
    temp = std::get<1>(self->impl->data);
    return temp.c_str();
}

int tuple2_equals(const struct Tuple2* self, int first, const char* second) {
    if (!second) return 0;
    return std::get<0>(self->impl->data) == first &&
           std::get<1>(self->impl->data) == second ? 1 : 0;
}

// Tuple3 C API implementation
struct Tuple3* tuple3_new(int first, double second, const char* third) {
    return new Tuple3(first, second, third);
}

void tuple3_delete(struct Tuple3* self) {
    delete self;
}

int tuple3_get_first(const struct Tuple3* self) {
    return std::get<0>(self->impl->data);
}

double tuple3_get_second(const struct Tuple3* self) {
    return std::get<1>(self->impl->data);
}

const char* tuple3_get_third(const struct Tuple3* self) {
    static thread_local std::string temp;
    temp = std::get<2>(self->impl->data);
    return temp.c_str();
}

int tuple3_equals(const struct Tuple3* self, int first, double second, const char* third) {
    if (!third) return 0;
    return std::get<0>(self->impl->data) == first &&
           std::get<1>(self->impl->data) == second &&
           std::get<2>(self->impl->data) == third ? 1 : 0;
}

// Tuple4 C API implementation
struct Tuple4* tuple4_new(int first, double second, const char* third, int fourth) {
    return new Tuple4(first, second, third, fourth);
}

void tuple4_delete(struct Tuple4* self) {
    delete self;
}

int tuple4_get_first(const struct Tuple4* self) {
    return std::get<0>(self->impl->data);
}

double tuple4_get_second(const struct Tuple4* self) {
    return std::get<1>(self->impl->data);
}

const char* tuple4_get_third(const struct Tuple4* self) {
    static thread_local std::string temp;
    temp = std::get<2>(self->impl->data);
    return temp.c_str();
}

int tuple4_get_fourth(const struct Tuple4* self) {
    return std::get<3>(self->impl->data);
}

// Helper functions
struct Tuple2* make_int_string_pair(int i, const char* s) {
    return new Tuple2(i, s);
}

struct Tuple3* make_int_double_string(int i, double d, const char* s) {
    return new Tuple3(i, d, s);
}
