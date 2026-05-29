#include "constexpr_basic.h"
#include <iostream>

namespace example {

// Compile-time computed fibonacci value
// fibonacci<10>() = 55
static constexpr int FIB_10 = fibonacci<10>();

}  // namespace example

// FFI implementations

int get_fibonacci_10() {
    constexpr int fib_10 = example::fibonacci<10>();
    std::cout << "get_fibonacci_10() called, returning compile-time computed value: "
              << fib_10 << std::endl;
    return fib_10;
}

int manhattan_distance(int x, int y) {
    const int dx = x > 0 ? x : -x;
    const int dy = y > 0 ? y : -y;
    return dx + dy;
}

int constexpr_sum_array(const int* arr, int size) {
    int sum = 0;
    for (int i = 0; i < size; ++i) {
        sum += arr[i];
    }
    return sum;
}

int constexpr_find_max(const int* arr, int size) {
    if (size <= 0) return 0;
    int max_val = arr[0];
    for (int i = 1; i < size; ++i) {
        if (arr[i] > max_val) {
            max_val = arr[i];
        }
    }
    return max_val;
}

int get_array_size() {
    return ARRAY_SIZE;
}
