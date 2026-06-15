#pragma once

#include <string>

class Base {
protected:
    std::string name;
public:
    Base(const char* n);
    virtual ~Base();
    virtual double area() const;
    const char* getName() const;
};

class Derived : public Base {
    double value;
public:
    Derived(double v);
    ~Derived() override;
    double area() const override;
    double getValue() const;
};
