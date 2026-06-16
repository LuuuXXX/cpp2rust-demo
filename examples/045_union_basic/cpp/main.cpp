#include "union_basic.h"
#include <iostream>

int main() {
    using namespace union_basic_ns;

    Variant v;
    v.set_int(42);
    std::cout << "variant int type=" << v.get_type() << " value=" << v.get_int() << "\n";
    v.set_float(2.5f);
    std::cout << "variant float type=" << v.get_type() << " value=" << v.get_float() << "\n";
    v.set_string("hi");
    std::cout << "variant string type=" << v.get_type() << " value=" << v.get_string() << "\n";

    IntFloatUnion u;
    u.set_int(7);
    std::cout << "union int=" << u.get_int() << "\n";
    u.set_float(1.5f);
    std::cout << "union float=" << u.get_float() << "\n";
    return 0;
}
