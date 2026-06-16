#include "string_basic.h"
#include <iostream>

int main() {
    using namespace string_basic_ns;

    MyString s("hello");
    std::cout << "empty=" << s.empty() << " length=" << s.length() << "\n";
    s.append(", world");
    std::cout << "after append=" << s.c_str() << " length=" << s.length() << "\n";
    std::cout << "at(1)=" << s.at(1) << " at(99)=" << static_cast<int>(s.at(99)) << "\n";
    std::cout << "compare hello=" << s.compare("hello") << "\n";
    std::cout << "find world=" << s.find("world") << " find missing=" << s.find("missing") << "\n";
    s.to_upper();
    std::cout << "to_upper=" << s.c_str() << "\n";
    return 0;
}
