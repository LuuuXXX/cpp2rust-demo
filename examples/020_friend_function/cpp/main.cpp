#include "friend_function.h"
#include <iostream>

int main() {
    using namespace friend_function_ns;
    MyClass a(10), b(3);
    std::cout << "a.getValue()=" << a.getValue() << std::endl;
    std::cout << "getSum(a,b)=" << getSum(a, b) << std::endl;
    std::cout << "getProduct(a,b)=" << getProduct(a, b) << std::endl;
    std::cout << "compare(a,b)=" << compare(a, b) << std::endl;
    a.setValue(3);
    std::cout << "compare(a,b) after setValue=" << compare(a, b) << std::endl;
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
