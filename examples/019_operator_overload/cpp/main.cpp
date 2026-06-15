#include "operator_overload.h"
#include <iostream>

int main() {
    using namespace operator_overload_ns;
    Number a(10), b(3);
    std::cout << "a+b=" << (a + b).value() << std::endl;
    std::cout << "a-b=" << (a - b).value() << std::endl;
    std::cout << "a*b=" << (a * b).value() << std::endl;
    std::cout << "a/b=" << (a / b).value() << std::endl;
    std::cout << "-a=" << (-a).value() << std::endl;
    std::cout << "compare(a,b)=" << a.compare(b) << std::endl;
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
