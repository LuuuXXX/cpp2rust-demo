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
