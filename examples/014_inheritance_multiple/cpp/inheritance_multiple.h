#pragma once

namespace inheritance_multiple_ns {

// 基类 1
class Base1 {
public:
    explicit Base1(int v);
    virtual ~Base1();
    int value1() const;
protected:
    int value1_;
};

// 基类 2
class Base2 {
public:
    explicit Base2(int v);
    virtual ~Base2();
    int value2() const;
protected:
    int value2_;
};

// 派生类：多继承 public Base1, public Base2
class Derived : public Base1, public Base2 {
public:
    Derived(int v1, int v2, int dv);
    ~Derived() override;
    int derived_value() const;
    int compute() const;   // 复用两个基类数据：value1_ + value2_ + derived_value_
private:
    int derived_value_;
};

// 锚点：让 detect_idiomatic_mode 走直出路径。
int inheritance_multiple_anchor();

} // namespace inheritance_multiple_ns
