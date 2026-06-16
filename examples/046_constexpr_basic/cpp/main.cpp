#include "constexpr_basic.h"
#include <iostream>

int main() {
    using namespace constexpr_basic_ns;

    ConstexprPoint p(3, 4);
    ConstexprPoint neg(-2, -5);

    std::cout << "p x=" << p.x()
              << " y=" << p.y()
              << " manhattan=" << p.manhattan_distance() << "\n";
    std::cout << "neg manhattan=" << neg.manhattan_distance() << "\n";
    std::cout << "fibonacci<10>()=" << fibonacci_10()
              << " array_size=" << array_size() << "\n";
    return 0;
}
