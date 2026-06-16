#include "placement_new.h"
#include <iostream>

int main() {
    using namespace placement_new_ns;

    Buffer buf(64);
    std::cout << "capacity=" << buf.capacity() << "\n";
    std::cout << "construct_at(0,42)=" << buf.construct_at(0, 42)
              << " value_at(0)=" << buf.value_at(0)
              << " size=" << buf.size() << "\n";
    std::cout << "construct_at(8,7)=" << buf.construct_at(8, 7)
              << " value_at(8)=" << buf.value_at(8) << "\n";

    ObjectArray arr(3);
    std::cout << "count=" << arr.count()
              << " element_size=" << arr.element_size() << "\n";
    for (int i = 0; i < arr.count(); ++i) arr.emplace(i, (i + 1) * 10);
    std::cout << "at(0)=" << arr.at(0)
              << " at(1)=" << arr.at(1)
              << " at(2)=" << arr.at(2) << "\n";
    return 0;
}
