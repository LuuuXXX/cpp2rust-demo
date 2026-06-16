#include "class_const.h"

int main() {
    class_const_ns::Calculator calc;
    calc.add(10);
    calc.add(5);
    calc.subtract(3);
    std::cout << "value=" << calc.value()
              << " history=" << calc.history_count() << std::endl;
    calc.clear();
    std::cout << "after clear value=" << calc.value()
              << " history=" << calc.history_count() << std::endl;
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
