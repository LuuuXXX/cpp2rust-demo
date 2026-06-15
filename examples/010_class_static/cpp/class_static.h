#pragma once
#include <iostream>

namespace class_static_ns {

// 演示静态成员：每个实例构造/析构维护一个跨实例共享的静态计数器。
// 静态方法以「自由函数式全限定调用」绑定，实例方法走 import_class! 直出。
class Counter {
public:
    Counter() : value_(0) {
        ++instance_count_;
        std::cout << "Counter() ctor, live=" << instance_count_ << std::endl;
    }
    ~Counter() {
        --instance_count_;
        std::cout << "~Counter() dtor, live=" << instance_count_ << std::endl;
    }

    int value() const { return value_; }
    void increment() { ++value_; }

    static int instance_count() { return instance_count_; }
    static void reset_instance_count() { instance_count_ = 0; }

private:
    int value_;
    static int instance_count_;
};

} // namespace class_static_ns
