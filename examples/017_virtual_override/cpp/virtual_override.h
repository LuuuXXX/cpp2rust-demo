#pragma once

namespace virtual_override_ns {

// 基类：虚函数 area() 默认 0
class Base {
public:
    Base();
    virtual ~Base();
    virtual double area() const;
};

// 派生类：以 override 关键字显式覆写 area()
class Derived : public Base {
public:
    explicit Derived(double v);
    ~Derived() override;
    double area() const override;   // value_ * value_
    double value() const;
private:
    double value_;
};

// 锚点：让 detect_idiomatic_mode 走直出路径。
int virtual_override_anchor();

} // namespace virtual_override_ns
