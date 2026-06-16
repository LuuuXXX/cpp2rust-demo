#include "virtual_pure.h"
#include <iostream>

int main() {
    using namespace virtual_pure_ns;
    Circle c(2.0);
    Rectangle r(3.0, 4.0);
    std::cout << "circle.area=" << c.area() << " radius=" << c.radius() << std::endl;
    std::cout << "rect.area=" << r.area() << std::endl;
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
