#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct D;

struct D* d_new(int a, int b, int c, int d);
void d_delete(struct D* self);

int d_getAValue(struct D* self);
int d_getBValue(struct D* self);
int d_getCValue(struct D* self);
int d_getDValue(struct D* self);
void d_compute(struct D* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
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

#endif
