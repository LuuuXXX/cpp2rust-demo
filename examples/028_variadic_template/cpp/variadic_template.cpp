#include "variadic_template.h"
#include <iostream>
#include <cstdarg>

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
