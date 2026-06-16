#include "unique_ptr.h"
#include <iostream>

int main() {
    using namespace unique_ptr_ns;

    UniqueBuffer buf(8);
    buf.fill('A');
    std::cout << "size=" << buf.size() << " at(0)=" << buf.at(0)
              << " use_count=" << buf.use_count() << std::endl;

    Processor proc;
    std::cout << "process: " << proc.process("hello") << std::endl;

    std::cout << "--- end main ---" << std::endl;
    return 0;
}
