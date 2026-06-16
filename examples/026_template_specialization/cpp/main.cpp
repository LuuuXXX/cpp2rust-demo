#include "template_specialization.h"
#include <iostream>

int main() {
    using namespace template_specialization_ns;

    IntHolder ih(42);
    std::cout << "int get=" << ih.get() << " describe=" << ih.describe() << std::endl;

    DoubleHolder dh(3.14159);
    std::cout << "double get=" << dh.get() << " describe=" << dh.describe() << std::endl;

    StringHolder sh("hello");
    std::cout << "string get=" << sh.get() << " describe=" << sh.describe() << std::endl;

    std::cout << "--- end main ---" << std::endl;
    return 0;
}
