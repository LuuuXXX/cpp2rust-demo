#pragma once

#include <iostream>

namespace friend_function_ns {

// 友元函数：在类体内内联定义的非成员友元，可访问私有成员 value_。
class MyClass {
public:
    explicit MyClass(int v);
    ~MyClass();

    int getValue() const;
    void setValue(int v);

    friend int getSum(const MyClass& a, const MyClass& b) {
        int sum = a.value_ + b.value_;
        std::cout << "Friend getSum: " << a.value_ << " + " << b.value_
                  << " = " << sum << std::endl;
        return sum;
    }
    friend int getProduct(const MyClass& a, const MyClass& b) {
        int product = a.value_ * b.value_;
        std::cout << "Friend getProduct: " << a.value_ << " * " << b.value_
                  << " = " << product << std::endl;
        return product;
    }
    friend int compare(const MyClass& a, const MyClass& b) {
        if (a.value_ < b.value_) { std::cout << "Friend compare: a < b\n"; return -1; }
        if (a.value_ > b.value_) { std::cout << "Friend compare: a > b\n"; return 1; }
        std::cout << "Friend compare: a == b\n";
        return 0;
    }

private:
    int value_;
};

// 锚点：触发 detect_idiomatic_mode 走直出路径。
int friend_function_anchor();

} // namespace friend_function_ns
