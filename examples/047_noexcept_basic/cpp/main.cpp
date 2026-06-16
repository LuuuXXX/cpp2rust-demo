#include "noexcept_basic.h"
#include <iostream>

int main() {
    using namespace noexcept_basic_ns;

    std::cout << "noexcept_add(2,3)=" << noexcept_add(2, 3) << "\n";
    std::cout << "noexcept_multiply(4,5)=" << noexcept_multiply(4, 5) << "\n";
    std::cout << "conditional_abs(-7)=" << conditional_abs(-7)
              << " conditional_abs(7)=" << conditional_abs(7) << "\n";
    std::cout << "safe_divide(10,2)=" << safe_divide(10, 2)
              << " safe_divide(10,0)=" << safe_divide(10, 0) << "\n";

    NoexceptMover mover(42);
    std::cout << "mover value=" << mover.get_value() << "\n";
    return 0;
}
