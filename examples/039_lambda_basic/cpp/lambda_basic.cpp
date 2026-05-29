#include "lambda_basic.h"
#include <iostream>
#include <functional>
#include <algorithm>

int add_impl(int a, int b) {
    std::cout << "add lambda called: " << a << " + " << b << std::endl;
    return a + b;
}

int multiply_impl(int a, int b) {
    std::cout << "multiply lambda called: " << a << " * " << b << std::endl;
    return a * b;
}

int max_impl(int a, int b) {
    std::cout << "max lambda called: " << a << " vs " << b << std::endl;
    return std::max(a, b);
}

int apply_operation(int a, int b, int (*op)(int, int)) {
    if (op) return op(a, b);
    return 0;
}

int apply_twice(int x, int (*op)(int, int)) {
    if (op) return op(op(x, x), x);
    return x;
}

struct LambdaWrapper* lambda_wrapper_new(int (*fn)(int, int)) {
    return new LambdaWrapper(fn);
}

void lambda_wrapper_delete(struct LambdaWrapper* self) {
    delete self;
}

int lambda_wrapper_call(const struct LambdaWrapper* self, int a, int b) {
    if (self) return self->impl->fn(a, b);
    return 0;
}

struct LambdaWrapper* make_add_lambda(void) {
    return new LambdaWrapper(add_impl);
}

struct LambdaWrapper* make_multiply_lambda(void) {
    return new LambdaWrapper(multiply_impl);
}

struct LambdaWrapper* make_max_lambda(void) {
    return new LambdaWrapper(max_impl);
}

struct StateLambda* state_lambda_new(int initial_value) {
    return new StateLambda(initial_value);
}

void state_lambda_delete(struct StateLambda* self) {
    delete self;
}

int state_lambda_apply(struct StateLambda* self, int delta) {
    if (self) return self->impl->adder(delta);
    return 0;
}

int state_lambda_get_value(const struct StateLambda* self) {
    if (self) return self->impl->value;
    return 0;
}

struct Comparator* comparator_new(int (*cmp)(int, int)) {
    return new Comparator(cmp);
}

struct Comparator* comparator_new_add(void) {
    return new Comparator(add_impl);
}

void comparator_delete(struct Comparator* self) {
    delete self;
}

int comparator_compare(const struct Comparator* self, int a, int b) {
    if (self) return self->impl->cmp(a, b);
    return 0;
}
