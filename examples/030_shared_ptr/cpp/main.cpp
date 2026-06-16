#include "shared_ptr.h"
#include <iostream>

int main() {
    using namespace shared_ptr_ns;

    SharedData data("TestData");
    std::cout << "name=" << data.name()
              << " use_count=" << data.use_count()
              << " expired=" << data.expired() << "\n";

    data.reset();
    std::cout << "after reset expired=" << data.expired() << "\n";

    Cache cache;
    int c1 = cache.store("key1");
    int c2 = cache.store("key2");
    std::cout << "store use_count=" << c1 << "," << c2
              << " size=" << cache.size() << "\n";
    cache.clear();
    std::cout << "after clear size=" << cache.size() << "\n";
    return 0;
}
