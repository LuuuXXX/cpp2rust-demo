#pragma once

#include <functional>

class LambdaWrapperImpl {
public:
    std::function<int(int, int)> fn;
    explicit LambdaWrapperImpl(int (*fn_ptr)(int, int)) : fn(fn_ptr) {}
    ~LambdaWrapperImpl() {}
};

class StateLambdaImpl {
public:
    int value;
    std::function<int(int)> adder;
    explicit StateLambdaImpl(int initial) : value(initial), adder([this](int delta) { return value += delta; }) {}
    ~StateLambdaImpl() {}
};

class ComparatorImpl {
public:
    std::function<int(int, int)> cmp;
    explicit ComparatorImpl(int (*cmp_fn)(int, int)) : cmp(cmp_fn) {}
    ~ComparatorImpl() {}
};

struct LambdaWrapper {
    LambdaWrapperImpl* impl;
    explicit LambdaWrapper(int (*fn)(int, int)) : impl(new LambdaWrapperImpl(fn)) {}
    ~LambdaWrapper() { delete impl; }
    int invoke(int a, int b) { return impl->fn(a, b); }
};

struct StateLambda {
    StateLambdaImpl* impl;
    explicit StateLambda(int initial_value) : impl(new StateLambdaImpl(initial_value)) {}
    ~StateLambda() { delete impl; }
    int get_value() const { return impl->value; }
    int add(int delta) { return impl->adder(delta); }
};

struct Comparator {
    ComparatorImpl* impl;
    explicit Comparator(int (*cmp)(int, int)) : impl(new ComparatorImpl(cmp)) {}
    ~Comparator() { delete impl; }
    int compare(int a, int b) const { return impl->cmp(a, b); }
};
