#pragma once

namespace constexpr_basic_ns {

class ConstexprPoint {
    int x_;
    int y_;
public:
    constexpr ConstexprPoint(int x, int y) : x_(x), y_(y) {}

    constexpr int x() const { return x_; }
    constexpr int y() const { return y_; }
    constexpr int manhattan_distance() const {
        return (x_ < 0 ? -x_ : x_) + (y_ < 0 ? -y_ : y_);
    }
};

template<int N>
constexpr int fibonacci() {
    return fibonacci<N - 1>() + fibonacci<N - 2>();
}

template<>
constexpr int fibonacci<0>() { return 0; }

template<>
constexpr int fibonacci<1>() { return 1; }

int fibonacci_10();
int array_size();

// 锚点：本单元可链接的非模板符号。
int constexpr_basic_anchor();

} // namespace constexpr_basic_ns
