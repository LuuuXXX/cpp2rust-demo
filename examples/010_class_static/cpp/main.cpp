#include "class_static.h"

int main() {
    using class_static_ns::Counter;
    Counter::reset_instance_count();
    std::cout << "initial live=" << Counter::instance_count() << std::endl;
    {
        Counter c1;
        Counter c2;
        c1.increment();
        c1.increment();
        c2.increment();
        std::cout << "live=" << Counter::instance_count()
                  << " c1=" << c1.value() << " c2=" << c2.value() << std::endl;
    }
    std::cout << "after scope live=" << Counter::instance_count() << std::endl;
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
