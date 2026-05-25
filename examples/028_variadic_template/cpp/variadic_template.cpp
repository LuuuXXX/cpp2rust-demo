#include "variadic_template.h"
#include <iostream>
#include <cstdarg>

// C-compatible wrapper functions
int sum_zero(void) {
    return SumCalculator::calculate_zero();
}

int sum_1(int a) {
    return SumCalculator::calculate_1(a);
}

int sum_2(int a, int b) {
    return SumCalculator::calculate_2(a, b);
}

int sum_3(int a, int b, int c) {
    return SumCalculator::calculate_3(a, b, c);
}

int sum_4(int a, int b, int c, int d) {
    return SumCalculator::calculate_4(a, b, c, d);
}

int sum_5(int a, int b, int c, int d, int e) {
    return SumCalculator::calculate_5(a, b, c, d, e);
}

double sum_double_2(double a, double b) {
    return SumCalculator::calculate_double_2(a, b);
}

double sum_double_3(double a, double b, double c) {
    return SumCalculator::calculate_double_3(a, b, c);
}

double sum_double_4(double a, double b, double c, double d) {
    return SumCalculator::calculate_double_4(a, b, c, d);
}

const char* sum_getFormat(int count) {
    return SumCalculator::get_format(count);
}

// SumCalculator implementation
int SumCalculator::calculate_zero() { return 0; }
int SumCalculator::calculate_1(int a) { return a; }
int SumCalculator::calculate_2(int a, int b) { return a + b; }
int SumCalculator::calculate_3(int a, int b, int c) { return a + b + c; }
int SumCalculator::calculate_4(int a, int b, int c, int d) { return a + b + c + d; }
int SumCalculator::calculate_5(int a, int b, int c, int d, int e) { return a + b + c + d + e; }
double SumCalculator::calculate_double_2(double a, double b) { return a + b; }
double SumCalculator::calculate_double_3(double a, double b, double c) { return a + b + c; }
double SumCalculator::calculate_double_4(double a, double b, double c, double d) { return a + b + c + d; }
const char* SumCalculator::get_format(int count) {
    switch (count) {
        case 0: return "sum()";
        case 1: return "sum(%d)";
        case 2: return "sum(%d, %d)";
        case 3: return "sum(%d, %d, %d)";
        case 4: return "sum(%d, %d, %d, %d)";
        case 5: return "sum(%d, %d, %d, %d, %d)";
        default: return "unknown";
    }
}
