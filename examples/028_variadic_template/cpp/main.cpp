#include "variadic_template.h"
#include <iostream>

int main() {
    using namespace variadic_template_ns;

    std::cout << "sum() = " << sum() << std::endl;
    std::cout << "sum(1,2,3) = " << sum(1, 2, 3) << std::endl;
    std::cout << "sum(1,2,3,4,5) = " << sum(1, 2, 3, 4, 5) << std::endl;
    std::cout << "sum(1.5,2.5,3.0) = " << sum(1.5, 2.5, 3.0) << std::endl;

    std::cout << "--- end main ---" << std::endl;
    return 0;
}
