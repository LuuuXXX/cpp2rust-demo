#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>
    #include <vector>

    class Calculator {
        int value;
        std::vector<int> history;
    public:
        Calculator() : value(0) {}
        ~Calculator() {}
        int getValue() const {
            return value;
        }
        int getHistoryCount() const {
            return static_cast<int>(history.size());
        }
        void add(int v) {
            history.push_back(v);
            value += v;
        }
        void subtract(int v) {
            history.push_back(-v);
            value -= v;
        }
        void clear() {
            history.clear();
            value = 0;
        }
    };

    Calculator* calculator_new() {
        return new Calculator();
    }

    void calculator_delete(Calculator* self) {
        delete self;
    }
#line 41
 struct Calculator_41;
#line 41
namespace hicc { template<> struct MethodsType<Calculator, void> { typedef Calculator_41 methods_type; }; }
#line 41
 struct Calculator_41 {
#line 41
typedef Calculator Self; typedef void SelfContainer; typedef Calculator_41 SelfMethods;
#line 43
static void _hicc_test_43() { int (Self::* _43)() const = &Self::getValue; (void)_43; }
#line 43
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getValue));
#line 46
static void _hicc_test_46() { int (Self::* _46)() const = &Self::getHistoryCount; (void)_46; }
#line 46
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getHistoryCount));
#line 49
static void _hicc_test_49() { void (Self::* _49)(int v) = &Self::add; (void)_49; }
#line 49
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(int v))&Self::add));
#line 52
static void _hicc_test_52() { void (Self::* _52)(int v) = &Self::subtract; (void)_52; }
#line 52
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(int v))&Self::subtract));
#line 55
static void _hicc_test_55() { void (Self::* _55)() = &Self::clear; (void)_55; }
#line 55
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 41
};
#line 61
EXPORT_METHODS_BEG(class_const) {
#line 65
static void _hicc_test_65() { Calculator* (* _65)() = &calculator_new; (void)_65; }
#line 65
EXPORT_METHOD_IN(void, ExportMethods, ((Calculator* (*)())&calculator_new));
#line 68
static void _hicc_test_68() { void (* _68)(Calculator* self) = &calculator_delete; (void)_68; }
#line 68
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Calculator* self))&calculator_delete));
#line 61
} EXPORT_METHODS_END();

