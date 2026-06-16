#pragma once
#include <string>
#include <iostream>

namespace class_basic_ns {

class Counter {
public:
    Counter() : count_(0), name_("anon") {}
    explicit Counter(const std::string& name) : count_(0), name_(name) {}

    void inc() { ++count_; }
    void inc_by(int delta) { count_ += delta; }
    void reset() { count_ = 0; }

    int count() const { return count_; }
    const std::string& name() const { return name_; }

private:
    int count_;
    std::string name_;
};

} // namespace class_basic_ns
