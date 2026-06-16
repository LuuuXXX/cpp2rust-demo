#include "constexpr_basic.h"

namespace constexpr_basic_ns {

int fibonacci_10() {
    return fibonacci<10>();
}

int array_size() {
    return 16;
}

int constexpr_basic_anchor() { return 0; }

} // namespace constexpr_basic_ns
