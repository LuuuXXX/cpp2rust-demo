#include "vector_basic.h"
#include <iostream>

int main() {
    using namespace vector_basic_ns;

    IntVector v;
    std::cout << "empty=" << v.empty() << "\n";
    v.reserve(8);
    for (int i = 0; i < 5; ++i) v.push_back(i * 10);
    std::cout << "size=" << v.size() << " sum=" << v.sum() << "\n";
    v.set(2, 999);
    std::cout << "get(2)=" << v.get(2) << "\n";
    v.pop_back();
    std::cout << "after pop_back size=" << v.size() << "\n";
    v.clear();
    std::cout << "after clear empty=" << v.empty() << "\n";

    StringVector sv;
    sv.push_back("alpha");
    sv.push_back("beta");
    std::cout << "sv size=" << sv.size()
              << " get(0)=" << sv.get(0)
              << " get(1)=" << sv.get(1) << "\n";
    return 0;
}
