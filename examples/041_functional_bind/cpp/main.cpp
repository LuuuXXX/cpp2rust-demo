#include "functional_bind.h"
#include <iostream>

int main() {
    using namespace functional_bind_ns;

    Adder adder(10);
    std::cout << "adder.add(5)=" << adder.add(5) << "\n";

    Multiplier multiplier(3);
    std::cout << "multiplier.multiply(4)=" << multiplier.multiply(4) << "\n";

    StringProcessor processor;
    processor.set_target("banana");
    std::cout << "count('a')=" << processor.count_char('a') << "\n";
    return 0;
}
