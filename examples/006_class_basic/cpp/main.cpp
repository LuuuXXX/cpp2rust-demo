#include "class_basic.h"

int main() {
    class_basic_ns::Counter c("demo");
    c.inc();
    c.inc();
    c.inc_by(10);
    std::cout << "name=" << c.name() << " count=" << c.count() << std::endl;
    c.reset();
    std::cout << "after reset, count=" << c.count() << std::endl;
    return 0;
}
