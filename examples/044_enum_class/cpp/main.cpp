#include "enum_class.h"
#include <iostream>

int main() {
    using namespace enum_class_ns;

    OperationResult result;
    result.set_error(3);
    result.set_state(1);
    result.set_flags(7);

    std::cout << "error=" << result.get_error()
              << " state=" << static_cast<int>(result.get_state())
              << " flags=" << result.get_flags() << "\n";
    std::cout << "combine_flags(1,2)=" << combine_flags(1, 2)
              << " has_execute=" << has_flag(result.get_flags(), 4)
              << " has_execute_in_read=" << has_flag(1, 4) << "\n";
    return 0;
}
