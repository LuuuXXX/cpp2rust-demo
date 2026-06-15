#include "virtual_basic.h"
#include <iostream>

int main() {
    using namespace virtual_basic_ns;
    Shape s;
    Circle c(2.0);
    std::cout << "shape.area=" << s.area() << std::endl;
    std::cout << "circle.area=" << c.area() << " radius=" << c.radius() << std::endl;
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
