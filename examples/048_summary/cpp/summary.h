#pragma once

namespace summary_ns {

class Counter {
    int count_;
public:
    Counter() : count_(0) {}

    void increment() { ++count_; }
    void decrement() { --count_; }
    int get() const { return count_; }
    void reset() { count_ = 0; }
};

int safe_add(int a, int b);
int max_size();

// 锚点：本单元可链接的非模板符号。
int summary_anchor();

} // namespace summary_ns
