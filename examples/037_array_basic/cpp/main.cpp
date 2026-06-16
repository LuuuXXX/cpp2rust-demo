#include "array_basic.h"
#include <iostream>

int main() {
    using namespace array_basic_ns;

    IntArray a;
    std::cout << "size=" << a.size() << " sum=" << a.sum() << "\n";
    for (int i = 0; i < a.size(); ++i) a.set(i, i * 10);
    std::cout << "after set sum=" << a.sum()
              << " min=" << a.min()
              << " max=" << a.max() << "\n";
    a.set(2, 999);
    std::cout << "get(2)=" << a.get(2)
              << " get(99)=" << a.get(99) << "\n";
    a.fill(7);
    std::cout << "after fill sum=" << a.sum()
              << " min=" << a.min()
              << " max=" << a.max() << "\n";
    return 0;
}
