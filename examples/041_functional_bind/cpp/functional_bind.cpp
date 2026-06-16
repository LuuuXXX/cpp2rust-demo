#include "functional_bind.h"

namespace functional_bind_ns {

Adder::Adder(int base) {
    add_ = std::bind(std::plus<int>(), base, std::placeholders::_1);
}

Multiplier::Multiplier(int factor) {
    mul_ = std::bind(std::multiplies<int>(), factor, std::placeholders::_1);
}

int StringProcessor::count_char(char ch) const {
    int count = 0;
    for (char c : target_) {
        if (c == ch) ++count;
    }
    return count;
}

int functional_bind_anchor() { return 0; }

} // namespace functional_bind_ns
