#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// Lambda 表达式示例
// 展示如何将 C++ lambda 通过 FFI 传递给 Rust

#include <stddef.h>

// Function pointer type
typedef int (*IntBinaryOp)(int, int);

// Functions using lambda
int apply_operation(int a, int b, int (*op)(int, int));
int apply_twice(int x, int (*op)(int, int));

// Lambda wrapper
struct LambdaWrapper;

struct LambdaWrapper* lambda_wrapper_new(int (*fn)(int, int));
void lambda_wrapper_delete(struct LambdaWrapper* self);

// Implementation functions (declared before factory functions to ensure correct ordering)
int add_impl(int a, int b);
int multiply_impl(int a, int b);
int max_impl(int a, int b);

// Predefined lambda factories
struct LambdaWrapper* make_add_lambda(void);
struct LambdaWrapper* make_multiply_lambda(void);
struct LambdaWrapper* make_max_lambda(void);

// State lambda (with captured state)
struct StateLambda;

struct StateLambda* state_lambda_new(int initial_value);
void state_lambda_delete(struct StateLambda* self);

int state_lambda_apply(struct StateLambda* self, int delta);
int state_lambda_get_value(const struct StateLambda* self);

// Comparator wrapper
struct Comparator;

struct Comparator* comparator_new(int (*cmp)(int, int));
struct Comparator* comparator_new_add(void);
void comparator_delete(struct Comparator* self);

int comparator_compare(const struct Comparator* self, int a, int b);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
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

#endif
