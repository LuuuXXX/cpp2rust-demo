#pragma once

#include <vector>
#include <string>

class IntVectorImpl {
public:
    std::vector<int> data;
    IntVectorImpl();
    ~IntVectorImpl();
};

class StringVectorImpl {
public:
    std::vector<std::string> data;
    StringVectorImpl();
    ~StringVectorImpl();
};

struct IntVector {
    IntVectorImpl* impl;
    explicit IntVector();
    ~IntVector();
    void push_back(int val) { impl->data.push_back(val); }
    int get(size_t i) const { return impl->data[i]; }
    void set(size_t i, int val) { impl->data[i] = val; }
    size_t size() const { return impl->data.size(); }
    bool empty() const { return impl->data.empty(); }
    size_t capacity() const { return impl->data.capacity(); }
    void reserve(size_t n) { impl->data.reserve(n); }
    int* data() { return impl->data.data(); }
    void clear() { impl->data.clear(); }
};

struct StringVector {
    StringVectorImpl* impl;
    explicit StringVector();
    ~StringVector();
    size_t size() const { return impl->data.size(); }
};
