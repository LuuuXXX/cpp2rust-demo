#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>

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

    A::A(int v) : a_value(v) {}

    A::~A() {}

    int A::getAValue() const {
        return a_value;
    }

    B::B(int a, int b) : A(a), b_value(b) {}

    B::~B() {}

    int B::getBValue() const {
        return b_value;
    }

    C::C(int a, int c) : A(a), c_value(c) {}

    C::~C() {}

    int C::getCValue() const {
        return c_value;
    }

    D::D(int a, int b, int c, int d) : A(a), B(a, b), C(a, c), d_value(d) {}

    D::~D() {}

    int D::getDValue() const {
        return d_value;
    }

    void D::compute() const {
        std::cout << "D::compute: a=" << a_value << " b=" << b_value
                  << " c=" << c_value << " d=" << d_value << std::endl;
        std::cout << "Sum: " << (a_value + b_value + c_value + d_value) << std::endl;
    }

    D* d_new(int a, int b, int c, int d) {
        return new D(a, b, c, d);
    }

    void d_delete(D* self) {
        delete self;
    }

    int d_getAValue(D* self) {
        std::cout << "Getting A value (virtual base - single instance)" << std::endl;
        return self->getAValue();
    }

    int d_getBValue(D* self) {
        return self->getBValue();
    }

    int d_getCValue(D* self) {
        return self->getCValue();
    }

    int d_getDValue(D* self) {
        return self->getDValue();
    }

    void d_compute(D* self) {
        self->compute();
    }
#line 110
 struct D_110;
#line 110
namespace hicc { template<> struct MethodsType<D, void> { typedef D_110 methods_type; }; }
#line 110
 struct D_110 {
#line 110
typedef D Self; typedef void SelfContainer; typedef D_110 SelfMethods;
#line 112
static void _hicc_test_112() { int (Self::* _112)() const = &Self::getBValue; (void)_112; }
#line 112
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getBValue));
#line 115
static void _hicc_test_115() { int (Self::* _115)() const = &Self::getCValue; (void)_115; }
#line 115
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getCValue));
#line 118
static void _hicc_test_118() { int (Self::* _118)() const = &Self::getDValue; (void)_118; }
#line 118
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getDValue));
#line 121
static void _hicc_test_121() { void (Self::* _121)() const = &Self::compute; (void)_121; }
#line 121
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)() const)&Self::compute));
#line 110
};
#line 127
EXPORT_METHODS_BEG(virtual_diamond) {
#line 131
static void _hicc_test_131() { D* (* _131)(int a, int b, int c, int d) = &d_new; (void)_131; }
#line 131
EXPORT_METHOD_IN(void, ExportMethods, ((D* (*)(int a, int b, int c, int d))&d_new));
#line 134
static void _hicc_test_134() { void (* _134)(D* self) = &d_delete; (void)_134; }
#line 134
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(D* self))&d_delete));
#line 137
static void _hicc_test_137() { int (* _137)(D* self) = &d_getAValue; (void)_137; }
#line 137
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(D* self))&d_getAValue));
#line 140
static void _hicc_test_140() { int (* _140)(D* self) = &d_getBValue; (void)_140; }
#line 140
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(D* self))&d_getBValue));
#line 143
static void _hicc_test_143() { int (* _143)(D* self) = &d_getCValue; (void)_143; }
#line 143
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(D* self))&d_getCValue));
#line 146
static void _hicc_test_146() { int (* _146)(D* self) = &d_getDValue; (void)_146; }
#line 146
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(D* self))&d_getDValue));
#line 149
static void _hicc_test_149() { void (* _149)(D* self) = &d_compute; (void)_149; }
#line 149
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(D* self))&d_compute));
#line 127
} EXPORT_METHODS_END();

