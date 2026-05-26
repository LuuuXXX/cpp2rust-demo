#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>

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

    Base1::Base1(int v) : value1(v) {}

    Base1::~Base1() {}

    int Base1::getValue1() const {
        return value1;
    }

    Base2::Base2(int v) : value2(v) {}

    Base2::~Base2() {}

    int Base2::getValue2() const {
        return value2;
    }

    Derived::Derived(int v1, int v2, int dv) : Base1(v1), Base2(v2), derived_value(dv) {}

    Derived::~Derived() {}

    int Derived::getDerivedValue() const {
        return derived_value;
    }

    void Derived::compute() const {
        std::cout << "Computing: " << value1 << " + " << value2 << " + " << derived_value
                  << " = " << (value1 + value2 + derived_value) << std::endl;
    }

    Derived* derived_new(int v1, int v2, int dv) {
        return new Derived(v1, v2, dv);
    }

    void derived_delete(Derived* self) {
        delete self;
    }
#line 71
 struct Derived_71;
#line 71
namespace hicc { template<> struct MethodsType<Derived, void> { typedef Derived_71 methods_type; }; }
#line 71
 struct Derived_71 {
#line 71
typedef Derived Self; typedef void SelfContainer; typedef Derived_71 SelfMethods;
#line 73
static void _hicc_test_73() { int (Self::* _73)() const = &Self::getValue1; (void)_73; }
#line 73
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getValue1));
#line 76
static void _hicc_test_76() { int (Self::* _76)() const = &Self::getValue2; (void)_76; }
#line 76
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getValue2));
#line 79
static void _hicc_test_79() { int (Self::* _79)() const = &Self::getDerivedValue; (void)_79; }
#line 79
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getDerivedValue));
#line 82
static void _hicc_test_82() { void (Self::* _82)() const = &Self::compute; (void)_82; }
#line 82
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)() const)&Self::compute));
#line 71
};
#line 88
EXPORT_METHODS_BEG(inheritance_multiple) {
#line 92
static void _hicc_test_92() { Derived* (* _92)(int v1, int v2, int dv) = &derived_new; (void)_92; }
#line 92
EXPORT_METHOD_IN(void, ExportMethods, ((Derived* (*)(int v1, int v2, int dv))&derived_new));
#line 95
static void _hicc_test_95() { void (* _95)(Derived* self) = &derived_delete; (void)_95; }
#line 95
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Derived* self))&derived_delete));
#line 88
} EXPORT_METHODS_END();

