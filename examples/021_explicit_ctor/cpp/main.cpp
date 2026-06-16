#include "explicit_ctor.h"
#include <iostream>

int main() {
    using namespace explicit_ctor_ns;
    Widget a(42);          // 隐式构造来源（int）
    Widget b(3.9);         // 显式构造（double，截断为 3）
    std::cout << "a.getValue()=" << a.getValue() << std::endl;
    std::cout << "b.getValue()=" << b.getValue() << std::endl;
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
