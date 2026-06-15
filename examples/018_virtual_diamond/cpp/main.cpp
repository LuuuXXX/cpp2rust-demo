#include "virtual_diamond.h"
#include <iostream>

int main() {
    using namespace virtual_diamond_ns;
    D d(1, 2, 3, 4);
    std::cout << "d_value=" << d.d_value()
              << " compute=" << d.compute() << std::endl;
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
