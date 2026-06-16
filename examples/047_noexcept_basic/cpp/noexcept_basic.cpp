#include "noexcept_basic.h"

namespace noexcept_basic_ns {

int throwing_divide(int a, int b) {
    if (b == 0) throw std::runtime_error("div by zero");
    return a / b;
}

int safe_divide(int a, int b) noexcept {
    try {
        return throwing_divide(a, b);
    } catch (...) {
        return -1;
    }
}

int noexcept_basic_anchor() { return 0; }

} // namespace noexcept_basic_ns
