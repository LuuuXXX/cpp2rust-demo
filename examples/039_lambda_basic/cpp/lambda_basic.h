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
int apply_operation(int a, int b, IntBinaryOp op);
int apply_twice(int x, IntBinaryOp op);

// Lambda wrapper
struct LambdaWrapper;

struct LambdaWrapper* lambda_wrapper_new(int (*fn)(int, int));
void lambda_wrapper_delete(struct LambdaWrapper* self);

int lambda_wrapper_call(const struct LambdaWrapper* self, int a, int b);

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
void comparator_delete(struct Comparator* self);

int comparator_compare(const struct Comparator* self, int a, int b);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
#include <functional>

class LambdaWrapperImpl {
public:
    std::function<int(int, int)> fn;
    explicit LambdaWrapperImpl(int (*fn_ptr)(int, int));
    ~LambdaWrapperImpl();
};

class StateLambdaImpl {
public:
    int value;
    std::function<int(int)> adder;
    explicit StateLambdaImpl(int initial);
    ~StateLambdaImpl();
};

class ComparatorImpl {
public:
    std::function<int(int, int)> cmp;
    explicit ComparatorImpl(int (*cmp_fn)(int, int));
    ~ComparatorImpl();
};

struct LambdaWrapper {
    LambdaWrapperImpl* impl;
    explicit LambdaWrapper(int (*fn)(int, int));
    ~LambdaWrapper();
};

struct StateLambda {
    StateLambdaImpl* impl;
    explicit StateLambda(int initial_value);
    ~StateLambda();
};

struct Comparator {
    ComparatorImpl* impl;
    explicit Comparator(int (*cmp)(int, int));
    ~Comparator();
};

#endif
