#include "inheritance_multiple.h"
#include <iostream>

int main() {
    using namespace inheritance_multiple_ns;
    Derived d(10, 20, 12);
    std::cout << "value1=" << d.value1()
              << " value2=" << d.value2()
              << " derived=" << d.derived_value()
              << " compute=" << d.compute() << std::endl;
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
