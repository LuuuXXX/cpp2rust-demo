#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct Base1;
struct Base2;
struct Derived;

struct Derived* derived_new(int v1, int v2, int dv);
void derived_delete(struct Derived* self);

int derived_getValue1(struct Derived* self);
int derived_getValue2(struct Derived* self);
int derived_getDerivedValue(struct Derived* self);
void derived_compute(struct Derived* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
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

#endif
