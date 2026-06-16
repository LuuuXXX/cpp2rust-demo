#include "template_class.h"
#include <iostream>

int main() {
    using namespace template_class_ns;

    IntStack is;
    is.push(1);
    is.push(2);
    is.push(3);
    std::cout << "IntStack size=" << is.size() << " top=" << is.top() << std::endl;
    is.pop();
    std::cout << "after pop: size=" << is.size() << " top=" << is.top() << std::endl;

    DoubleStack ds;
    ds.push(3.14);
    ds.push(2.71);
    std::cout << "DoubleStack size=" << ds.size() << " top=" << ds.top() << std::endl;
    std::cout << "empty? " << (ds.empty() ? "yes" : "no") << std::endl;

    std::cout << "--- end main ---" << std::endl;
    return 0;
}
