#pragma once

#include <stdexcept>

namespace noexcept_basic_ns {

int noexcept_add(int a, int b) noexcept;
int noexcept_multiply(int a, int b) noexcept;
int conditional_abs(int x) noexcept;

class NoexceptMover {
    int value_;
public:
    explicit NoexceptMover(int v) noexcept : value_(v) {}
    NoexceptMover(const NoexceptMover&) = delete;
    NoexceptMover& operator=(const NoexceptMover&) = delete;
    NoexceptMover(NoexceptMover&& o) noexcept : value_(o.value_) { o.value_ = 0; }
    NoexceptMover& operator=(NoexceptMover&&) noexcept = default;
    int get_value() const noexcept { return value_; }
};

int throwing_divide(int a, int b);
int safe_divide(int a, int b) noexcept;

// 锚点：本单元可链接的非模板符号。
int noexcept_basic_anchor();

} // namespace noexcept_basic_ns
