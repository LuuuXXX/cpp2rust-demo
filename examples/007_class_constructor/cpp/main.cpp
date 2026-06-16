#include "class_constructor.h"

int main() {
    {
        class_ctor_ns::Widget w1;
        class_ctor_ns::Widget w2(99);
        class_ctor_ns::Widget w3("named", 7);
        std::cout << "w1: " << w1.name() << "/" << w1.value() << std::endl;
        std::cout << "w2: " << w2.name() << "/" << w2.value() << std::endl;
        std::cout << "w3: " << w3.name() << "/" << w3.value() << std::endl;
    }
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
