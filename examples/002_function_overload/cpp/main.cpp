#include "function_overload.h"
#include <iostream>

int main() {
    std::cout << function_overload_ns::add_int(1, 2) << std::endl;
    std::cout << function_overload_ns::add_double(1.5, 2.5) << std::endl;
    std::cout << function_overload_ns::add_strings("Hello", " World") << std::endl;
    std::cout << function_overload_ns::sum3(1, 2, 3) << std::endl;
    return 0;
}
