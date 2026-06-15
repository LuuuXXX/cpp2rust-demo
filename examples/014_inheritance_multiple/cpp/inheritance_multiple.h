#pragma once

class Base1 {
protected:
    int value1;
public:
    Base1(int v);
    virtual ~Base1();
    int getValue1() const;
};

class Base2 {
protected:
    int value2;
public:
    Base2(int v);
    virtual ~Base2();
    int getValue2() const;
};

class Derived : public Base1, public Base2 {
private:
    int derived_value;
public:
    Derived(int v1, int v2, int dv);
    ~Derived();
    int getDerivedValue() const;
    void compute() const;
};
