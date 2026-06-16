#include "custom_deleter.h"
#include <iostream>

int main() {
    using namespace custom_deleter_ns;

    int before = cleanup_count();
    {
        ManagedResource res("logfile.txt");
        std::cout << "name=" << res.name()
                  << " released=" << res.released() << "\n";
        res.release();
        std::cout << "after release released=" << res.released() << "\n";
    }
    std::cout << "cleanup_count delta=" << (cleanup_count() - before) << "\n";
    return 0;
}
