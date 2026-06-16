#include "template_instantiation.h"
#include <iostream>

int main() {
    using namespace template_instantiation_ns;

    IntMatrix im(2, 3);
    im.set(0, 0, 1);
    im.set(1, 2, 6);
    std::cout << "IntMatrix " << im.rows() << "x" << im.cols()
              << " get(1,2)=" << im.get(1, 2) << std::endl;
    im.print();

    DoubleMatrix dm(2, 2);
    dm.set(0, 0, 1.5);
    dm.set(1, 1, 2.5);
    std::cout << "DoubleMatrix get(1,1)=" << dm.get(1, 1) << std::endl;

    std::cout << "--- end main ---" << std::endl;
    return 0;
}
