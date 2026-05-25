#include "function_overload.h"
#include <iostream>
#include <cstring>

int add_int(int a, int b) {
    std::cout << "add_int(" << a << ", " << b << ")" << std::endl;
    return a + b;
}

double add_double(double a, double b) {
    std::cout << "add_double(" << a << ", " << b << ")" << std::endl;
    return a + b;
}

const char* add_strings(const char* a, const char* b) {
    std::cout << "add_strings(\"" << a << "\", \"" << b << "\")" << std::endl;
    static char result[256];
    snprintf(result, sizeof(result), "%s%s", a, b);
    return result;
}

int sum3(int a, int b, int c) {
    std::cout << "sum3(" << a << ", " << b << ", " << c << ")" << std::endl;
    return a + b + c;
}
