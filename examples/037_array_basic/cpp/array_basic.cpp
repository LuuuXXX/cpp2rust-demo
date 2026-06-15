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
