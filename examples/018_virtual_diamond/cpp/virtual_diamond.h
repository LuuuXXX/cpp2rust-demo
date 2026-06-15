#pragma once

class A {
protected:
    int a_value;
public:
    A(int v);
    virtual ~A();
    int getAValue() const;
};

class B : virtual public A {
protected:
    int b_value;
public:
    B(int a, int b);
    virtual ~B();
    int getBValue() const;
};

class C : virtual public A {
protected:
    int c_value;
public:
    C(int a, int c);
    virtual ~C();
    int getCValue() const;
};

class D : public B, public C {
private:
    int d_value;
public:
    D(int a, int b, int c, int d);
    ~D();
    int getDValue() const;
    void compute() const;
};
