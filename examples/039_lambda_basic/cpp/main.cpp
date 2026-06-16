#include "lambda_basic.h"
#include <iostream>

int main() {
    using namespace lambda_basic_ns;

    Operation add(0), mul(1), mx(2);
    std::cout << "add(3,4)=" << add.apply(3, 4) << "\n";
    std::cout << "mul(3,4)=" << mul.apply(3, 4) << "\n";
    std::cout << "max(3,4)=" << mx.apply(3, 4) << "\n";

    Accumulator acc(10);
    std::cout << "acc.apply(5)=" << acc.apply(5)
              << " apply(3)=" << acc.apply(3)
              << " value=" << acc.value() << "\n";
    return 0;
}
