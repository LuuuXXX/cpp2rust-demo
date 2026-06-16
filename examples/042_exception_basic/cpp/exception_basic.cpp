#include "exception_basic.h"

namespace exception_basic_ns {

Calculator::Calculator() : last_error_(0) {}

int Calculator::last_error() const {
    return last_error_;
}

void Calculator::clear_error() {
    last_error_ = 0;
}

int Calculator::has_error() const {
    return last_error_ != 0 ? 1 : 0;
}

int Calculator::divide(int a, int b) {
    clear_error();
    try {
        if (b == 0) {
            throw std::runtime_error("division by zero");
        }
        return a / b;
    } catch (const std::runtime_error&) {
        last_error_ = 3;
        return 0;
    }
}

int Calculator::parse_int(const char* s) {
    clear_error();
    try {
        return std::stoi(s ? s : "");
    } catch (const std::invalid_argument&) {
        last_error_ = 1;
        return 0;
    } catch (const std::out_of_range&) {
        last_error_ = 2;
        return 0;
    }
}

int exception_basic_anchor() { return 0; }

} // namespace exception_basic_ns
