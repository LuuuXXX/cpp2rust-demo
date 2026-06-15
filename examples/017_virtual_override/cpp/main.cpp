#include "virtual_override.h"
#include <iostream>

int main() {
    using namespace virtual_override_ns;
    Base b;
    Derived d(6.0);
    std::cout << "base.area=" << b.area() << std::endl;
    std::cout << "derived.area=" << d.area() << " value=" << d.value() << std::endl;
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
