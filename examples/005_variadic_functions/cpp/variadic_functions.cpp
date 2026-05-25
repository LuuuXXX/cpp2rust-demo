#include "variadic_functions.h"
#include <cstdarg>
#include <cstdio>

int sum(int count, ...) {
    va_list args;
    va_start(args, count);
    int total = 0;
    for (int i = 0; i < count; ++i) {
        total += va_arg(args, int);
    }
    va_end(args);
    return total;
}

int print_formatted(const char* format, ...) {
    va_list args;
    va_start(args, format);
    int result = vprintf(format, args);
    va_end(args);
    return result;
}

// Wrapper functions for FFI (Rust cannot call variadic functions directly)
int sum_3(int a, int b, int c) {
    return sum(3, a, b, c);
}

int sum_5(int a, int b, int c, int d, int e) {
    return sum(5, a, b, c, d, e);
}
