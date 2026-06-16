#include "std_function.h"

namespace std_function_ns {

Callback::Callback(int kind) {
    switch (kind) {
        case 1:
            fn_ = [](int v) { return v * 3; };
            break;
        case 2:
            fn_ = [](int v) { return -v; };
            break;
        default:
            fn_ = [](int v) { return v * 2; };
            break;
    }
}

void Pipeline::add(int kind) {
    switch (kind) {
        case 1:
            fns_.push_back([](int v) { return v * 3; });
            break;
        case 2:
            fns_.push_back([](int v) { return -v; });
            break;
        default:
            fns_.push_back([](int v) { return v * 2; });
            break;
    }
}

int Pipeline::run(int v) const {
    int result = v;
    for (const auto& fn : fns_) result = fn(result);
    return result;
}

int std_function_anchor() { return 0; }

} // namespace std_function_ns
