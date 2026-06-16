#include "lambda_basic.h"

namespace lambda_basic_ns {

Operation::Operation(int kind) {
    switch (kind) {
        case 1:
            fn_ = [](int a, int b) { return a * b; };
            break;
        case 2:
            fn_ = [](int a, int b) { return a > b ? a : b; };
            break;
        default:
            fn_ = [](int a, int b) { return a + b; };
            break;
    }
}

Accumulator::Accumulator(int initial) : value_(initial) {
    // 捕获 this 的状态 lambda：每次调用把 delta 累加进 value_。
    adder_ = [this](int delta) { return value_ += delta; };
}

int lambda_basic_anchor() { return 0; }

} // namespace lambda_basic_ns
