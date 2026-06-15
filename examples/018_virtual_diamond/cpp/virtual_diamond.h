#pragma once

namespace virtual_diamond_ns {

// 顶点基类 A
class A {
public:
    explicit A(int v);
    virtual ~A();
    int a_value() const;
protected:
    int a_value_;
};

// B 虚继承 A
class B : virtual public A {
public:
    B(int a, int b);
    ~B() override;
    int b_value() const;
protected:
    int b_value_;
};

// C 虚继承 A
class C : virtual public A {
public:
    C(int a, int c);
    ~C() override;
    int c_value() const;
protected:
    int c_value_;
};

// D 多继承 B、C：菱形继承，A 子对象唯一（虚继承）
class D : public B, public C {
public:
    D(int a, int b, int c, int d);
    ~D() override;
    int d_value() const;
    int compute() const;   // a_value_ + b_value_ + c_value_ + d_value_
private:
    int d_value_;
};

// 锚点：让 detect_idiomatic_mode 走直出路径。
int virtual_diamond_anchor();

} // namespace virtual_diamond_ns
