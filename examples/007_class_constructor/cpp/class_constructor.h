#pragma once
#include <string>
#include <iostream>

namespace class_ctor_ns {

class Widget {
public:
    Widget() : name_("default"), value_(0) {
        std::cout << "Widget() ctor" << std::endl;
    }
    explicit Widget(int v) : name_("int"), value_(v) {
        std::cout << "Widget(int) ctor, v=" << v << std::endl;
    }
    Widget(std::string n, int v) : name_(std::move(n)), value_(v) {
        std::cout << "Widget(string,int) ctor, n=" << name_ << " v=" << v << std::endl;
    }
    ~Widget() {
        std::cout << "~Widget() dtor, name=" << name_ << std::endl;
    }

    const std::string& name() const { return name_; }
    int value() const { return value_; }

private:
    std::string name_;
    int value_;
};

} // namespace class_ctor_ns
