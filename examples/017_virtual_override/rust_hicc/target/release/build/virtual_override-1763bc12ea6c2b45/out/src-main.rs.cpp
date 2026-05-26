#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <string>
    #include <iostream>
    #include <cstring>

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

    Base::Base(const char* n) : name(n) {}

    Base::~Base() {}

    double Base::area() const {
        return 0.0;
    }

    const char* Base::getName() const {
        return name.c_str();
    }

    Derived::Derived(double v) : Base("Derived"), value(v) {}

    Derived::~Derived() {}

    double Derived::area() const {
        return value * value;
    }

    double Derived::getValue() const {
        return value;
    }

    Base* base_create(int type) {
        if (type == 0) {
            std::cout << "Creating Base" << std::endl;
            return new Base("Base");
        } else {
            std::cout << "Creating Derived (as Base*)" << std::endl;
            return new Derived(42.0);
        }
    }

    void base_delete(Base* self) {
        delete self;
    }

    Derived* derived_new(double value) {
        return new Derived(value);
    }

    void derived_delete(Derived* self) {
        delete self;
    }
#line 73
 struct Base_73;
#line 73
namespace hicc { template<> struct MethodsType<Base, void> { typedef Base_73 methods_type; }; }
#line 73
 struct Base_73 {
#line 73
typedef Base Self; typedef void SelfContainer; typedef Base_73 SelfMethods;
#line 75
static void _hicc_test_75() { double (Self::* _75)() const = &Self::area; (void)_75; }
#line 75
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((double (Self::*)() const)&Self::area));
#line 78
static void _hicc_test_78() { const char* (Self::* _78)() const = &Self::getName; (void)_78; }
#line 78
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::getName));
#line 73
};
#line 84
 struct Derived_84;
#line 84
namespace hicc { template<> struct MethodsType<Derived, void> { typedef Derived_84 methods_type; }; }
#line 84
 struct Derived_84 {
#line 84
typedef Derived Self; typedef void SelfContainer; typedef Derived_84 SelfMethods;
#line 86
static void _hicc_test_86() { double (Self::* _86)() const = &Self::area; (void)_86; }
#line 86
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((double (Self::*)() const)&Self::area));
#line 89
static void _hicc_test_89() { double (Self::* _89)() const = &Self::getValue; (void)_89; }
#line 89
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((double (Self::*)() const)&Self::getValue));
#line 84
};
#line 95
EXPORT_METHODS_BEG(virtual_override) {
#line 100
static void _hicc_test_100() { Base* (* _100)(int type) = &base_create; (void)_100; }
#line 100
EXPORT_METHOD_IN(void, ExportMethods, ((Base* (*)(int type))&base_create));
#line 103
static void _hicc_test_103() { void (* _103)(Base* self) = &base_delete; (void)_103; }
#line 103
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Base* self))&base_delete));
#line 106
static void _hicc_test_106() { Derived* (* _106)(double value) = &derived_new; (void)_106; }
#line 106
EXPORT_METHOD_IN(void, ExportMethods, ((Derived* (*)(double value))&derived_new));
#line 109
static void _hicc_test_109() { void (* _109)(Derived* self) = &derived_delete; (void)_109; }
#line 109
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Derived* self))&derived_delete));
#line 95
} EXPORT_METHODS_END();

