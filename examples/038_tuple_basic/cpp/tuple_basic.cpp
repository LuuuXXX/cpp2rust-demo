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
