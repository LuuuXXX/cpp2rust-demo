#include "inline_functions.h"
#include <iostream>

int main() {
    std::cout << inline_functions_ns::min(10, 20) << std::endl;
    std::cout << inline_functions_ns::max(10, 20) << std::endl;
    std::cout << inline_functions_ns::min_v2(10, 20) << std::endl;
    std::cout << inline_functions_ns::max_v2(10, 20) << std::endl;
    return 0;
}
