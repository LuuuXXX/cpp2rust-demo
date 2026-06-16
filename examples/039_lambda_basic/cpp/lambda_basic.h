#pragma once

#include <functional>

namespace lambda_basic_ns {

// Operation：内部持有由 lambda 构造的 std::function，按 kind 选择运算。
// 演示 C++ lambda 作为可调用对象被类捕获持有。hicc 直出无需把函数指针跨 FFI 传递。
class Operation {
    std::function<int(int, int)> fn_;
public:
    // kind: 0=add, 1=multiply, 2=max
    explicit Operation(int kind);
    Operation(const Operation&) = delete;
    Operation& operator=(const Operation&) = delete;

    int apply(int a, int b) const { return fn_(a, b); }
};

// Accumulator：状态 lambda（捕获 this），apply 把 delta 累加进 value，演示闭包捕获状态。
class Accumulator {
    int value_;
    std::function<int(int)> adder_;
public:
    explicit Accumulator(int initial);
    Accumulator(const Accumulator&) = delete;
    Accumulator& operator=(const Accumulator&) = delete;

    int apply(int delta) { return adder_(delta); }
    int value() const { return value_; }
};

// 锚点：本单元可链接的非模板符号。
int lambda_basic_anchor();

} // namespace lambda_basic_ns
