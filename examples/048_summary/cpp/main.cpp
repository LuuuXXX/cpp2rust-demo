#include "summary.h"
#include <iostream>

int main() {
    using namespace summary_ns;

    Counter counter;
    std::cout << "initial=" << counter.get() << "\n";
    counter.increment();
    counter.increment();
    counter.increment();
    std::cout << "after increment x3=" << counter.get() << "\n";
    counter.decrement();
    std::cout << "after decrement=" << counter.get() << "\n";
    counter.reset();
    std::cout << "after reset=" << counter.get() << "\n";
    std::cout << "safe_add(2,3)=" << safe_add(2, 3)
              << " max_size=" << max_size() << "\n";
    return 0;
}
