#pragma once
#include <vector>
#include <iostream>

namespace class_const_ns {

// 演示 const 成员函数：const 方法（value/history_count）保证不修改对象状态，
// 映射为 Rust 的 `&self`；非 const 方法（add/subtract/clear）映射为 `&mut self`。
class Calculator {
public:
    Calculator() : value_(0) {
        std::cout << "Calculator() ctor" << std::endl;
    }
    ~Calculator() {
        std::cout << "~Calculator() dtor" << std::endl;
    }

    // const 成员函数（只读）。
    int value() const { return value_; }
    int history_count() const { return static_cast<int>(history_.size()); }

    // 非 const 成员函数（可变）。
    void add(int v) {
        value_ += v;
        history_.push_back(value_);
    }
    void subtract(int v) {
        value_ -= v;
        history_.push_back(value_);
    }
    void clear() {
        value_ = 0;
        history_.clear();
    }

private:
    int value_;
    std::vector<int> history_;
};

} // namespace class_const_ns
