#include "class_volatile.h"

int main() {
    class_volatile_ns::HardwareDevice dev;
    std::cout << std::hex
              << "status=0x" << dev.read_status()
              << " data=0x" << dev.read_data() << std::endl;
    dev.init();
    std::cout << "after init status=0x" << dev.read_status()
              << " data=0x" << dev.read_data() << std::endl;
    dev.reset();
    std::cout << "after reset status=0x" << dev.read_status() << std::dec << std::endl;
    std::cout << "--- end main ---" << std::endl;
    return 0;
}
