#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>

    class MyClass {
        int secret_value;
        friend int friend_function_getSum(const MyClass* a, const MyClass* b);
        friend int friend_function_getProduct(const MyClass* a, const MyClass* b);
        friend int friend_function_compare(const MyClass* a, const MyClass* b);
    public:
        MyClass(int v);
        ~MyClass();
        int getValue() const;
        void setValue(int v);
    };

    MyClass* myclass_new(int secret_value) {
        return new MyClass(secret_value);
    }

    void myclass_delete(MyClass* self) {
        delete self;
    }

    int myclass_getValue(MyClass* self) {
        return self->getValue();
    }

    void myclass_setValue(MyClass* self, int value) {
        self->setValue(value);
    }

    int friend_function_getSum(const MyClass* a, const MyClass* b) {
        int sum = a->secret_value + b->secret_value;
        std::cout << "Friend function getSum: " << a->secret_value
                  << " + " << b->secret_value << " = " << sum << std::endl;
        return sum;
    }

    int friend_function_getProduct(const MyClass* a, const MyClass* b) {
        int product = a->secret_value * b->secret_value;
        std::cout << "Friend function getProduct: " << a->secret_value
                  << " * " << b->secret_value << " = " << product << std::endl;
        return product;
    }

    int friend_function_compare(const MyClass* a, const MyClass* b) {
        if (a->secret_value < b->secret_value) {
            std::cout << "Friend function compare: a < b" << std::endl;
            return -1;
        } else if (a->secret_value > b->secret_value) {
            std::cout << "Friend function compare: a > b" << std::endl;
            return 1;
        } else {
            std::cout << "Friend function compare: a == b" << std::endl;
            return 0;
        }
    }

    MyClass::MyClass(int v) : secret_value(v) {}
    MyClass::~MyClass() {}
    int MyClass::getValue() const { return secret_value; }
    void MyClass::setValue(int v) { secret_value = v; }
#line 66
 struct MyClass_66;
#line 66
namespace hicc { template<> struct MethodsType<MyClass, void> { typedef MyClass_66 methods_type; }; }
#line 66
 struct MyClass_66 {
#line 66
typedef MyClass Self; typedef void SelfContainer; typedef MyClass_66 SelfMethods;
#line 68
static void _hicc_test_68() { int (Self::* _68)() const = &Self::getValue; (void)_68; }
#line 68
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getValue));
#line 71
static void _hicc_test_71() { void (Self::* _71)(int v) = &Self::setValue; (void)_71; }
#line 71
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(int v))&Self::setValue));
#line 66
};
#line 77
EXPORT_METHODS_BEG(friend_function) {
#line 81
static void _hicc_test_81() { MyClass* (* _81)(int secret_value) = &myclass_new; (void)_81; }
#line 81
EXPORT_METHOD_IN(void, ExportMethods, ((MyClass* (*)(int secret_value))&myclass_new));
#line 84
static void _hicc_test_84() { void (* _84)(MyClass* self) = &myclass_delete; (void)_84; }
#line 84
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(MyClass* self))&myclass_delete));
#line 87
static void _hicc_test_87() { int (* _87)(MyClass* self) = &myclass_getValue; (void)_87; }
#line 87
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(MyClass* self))&myclass_getValue));
#line 90
static void _hicc_test_90() { void (* _90)(MyClass* self, int value) = &myclass_setValue; (void)_90; }
#line 90
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(MyClass* self, int value))&myclass_setValue));
#line 93
static void _hicc_test_93() { int (* _93)(const MyClass* a, const MyClass* b) = &friend_function_getSum; (void)_93; }
#line 93
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(const MyClass* a, const MyClass* b))&friend_function_getSum));
#line 96
static void _hicc_test_96() { int (* _96)(const MyClass* a, const MyClass* b) = &friend_function_getProduct; (void)_96; }
#line 96
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(const MyClass* a, const MyClass* b))&friend_function_getProduct));
#line 99
static void _hicc_test_99() { int (* _99)(const MyClass* a, const MyClass* b) = &friend_function_compare; (void)_99; }
#line 99
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(const MyClass* a, const MyClass* b))&friend_function_compare));
#line 77
} EXPORT_METHODS_END();

