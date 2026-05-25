#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct Base;
struct Derived;

struct Base* base_create(int type);
void base_delete(struct Base* self);

double base_area(struct Base* self);
const char* base_getName(struct Base* self);

struct Derived* derived_new(double value);
void derived_delete(struct Derived* self);
double derived_getValue(struct Derived* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
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

#endif
