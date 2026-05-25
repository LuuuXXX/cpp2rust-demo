#pragma once

#include <cstddef>

#ifdef __cplusplus
extern "C" {
#endif

// Compile-time array size constant
static const int ARRAY_SIZE = 10;

// FFI functions that use compile-time computed values
int get_fibonacci_10(void);
int manhattan_distance(int x, int y);
int constexpr_sum_array(const int* arr, int size);
int constexpr_find_max(const int* arr, int size);
int get_array_size(void);

#ifdef __cplusplus
}
#endif

// C++ only: constexpr utilities (not part of FFI boundary)
#ifdef __cplusplus

namespace example {

// Compile-time fibonacci calculation
template<int N>
constexpr int fibonacci() {
    if constexpr (N <= 1) {
        return N;
    } else {
        return fibonacci<N - 1>() + fibonacci<N - 2>();
    }
}

// Compile-time point with constexpr constructor and methods
struct ConstexprPoint {
    int x;
    int y;

    constexpr ConstexprPoint(int x, int y) : x(x), y(y) {}

    constexpr int manhattan_distance() const {
        return (x > 0 ? x : -x) + (y > 0 ? y : -y);
    }
};

}  // namespace example

#endif  // __cplusplus
