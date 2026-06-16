#include "template_function.h"
#include <iostream>

int main() {
    using namespace template_function_ns;
    int a = 10, b = 20;
    do_swap(&a, &b);
    std::cout << "swap int: a=" << a << " b=" << b << std::endl;

    double x = 3.14, y = 2.71;
    do_swap(&x, &y);
    std::cout << "swap double: x=" << x << " y=" << y << std::endl;

    std::cout << "max_value(3,7)=" << max_value(3, 7) << std::endl;
    std::cout << "max_value(2.5,1.5)=" << max_value(2.5, 1.5) << std::endl;
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
