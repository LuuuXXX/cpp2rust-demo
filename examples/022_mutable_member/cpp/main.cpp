#include "mutable_member.h"
#include <iostream>

int main() {
    using namespace mutable_member_ns;
    DataFetcher f(100);
    std::cout << "fetch=" << f.fetch() << std::endl;   // 101
    std::cout << "fetch=" << f.fetch() << std::endl;   // 102
    std::cout << "accessCount=" << f.accessCount() << std::endl; // 2
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
