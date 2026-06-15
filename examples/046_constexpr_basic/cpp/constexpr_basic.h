#pragma once

#include <cstddef>

#ifdef __cplusplus

namespace example {

template<int N>
constexpr int fibonacci() {
    if constexpr (N <= 1) {
        return N;
    } else {
        return fibonacci<N - 1>() + fibonacci<N - 2>();
    }
}

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
