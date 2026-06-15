#pragma once

#include <array>
#include <string>

class IntArray5Impl {
public:
    std::array<int, 5> data;
    IntArray5Impl();
    explicit IntArray5Impl(const int* values);
    ~IntArray5Impl();
};

class DoubleArray3Impl {
public:
    std::array<double, 3> data;
    DoubleArray3Impl();
    explicit DoubleArray3Impl(const double* values);
    ~DoubleArray3Impl();
};

class StringArray4Impl {
public:
    std::array<std::string, 4> data;
    bool initialized[4];
    StringArray4Impl();
    ~StringArray4Impl();
};

struct IntArray5 {
    IntArray5Impl* impl;
    explicit IntArray5();
    explicit IntArray5(const int* values);
    ~IntArray5();
    size_t size() const { return impl->data.size(); }
    bool empty() const { return impl->data.empty(); }
    void set(size_t i, int val) { impl->data[i] = val; }
    int get(size_t i) const { return impl->data[i]; }
    int at(size_t i) const { return impl->data.at(i); }
    int* data() { return impl->data.data(); }
};

struct DoubleArray3 {
    DoubleArray3Impl* impl;
    explicit DoubleArray3();
    explicit DoubleArray3(const double* values);
    ~DoubleArray3();
    size_t size() const { return impl->data.size(); }
};

struct StringArray4 {
    StringArray4Impl* impl;
    explicit StringArray4();
    ~StringArray4();
    size_t size() const { return impl->data.size(); }
};
