#include "noexcept_basic.h"

namespace noexcept_basic_ns {

int noexcept_add(int a, int b) noexcept { return a + b; }
int noexcept_multiply(int a, int b) noexcept { return a * b; }
int conditional_abs(int x) noexcept { return x < 0 ? -x : x; }

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
