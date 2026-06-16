#include "std_function.h"
#include <iostream>

int main() {
    using namespace std_function_ns;

    Callback dbl(0), tri(1), neg(2);
    std::cout << "double(5)=" << dbl.invoke(5) << "\n";
    std::cout << "triple(5)=" << tri.invoke(5) << "\n";
    std::cout << "negate(5)=" << neg.invoke(5) << "\n";

    Pipeline p;
    p.add(0);
    p.add(1);
    std::cout << "pipeline size=" << p.size() << " run(2)=" << p.run(2) << "\n";
    return 0;
}
