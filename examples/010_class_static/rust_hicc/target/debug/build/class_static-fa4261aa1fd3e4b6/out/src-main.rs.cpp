#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>

    class Counter {
        int value;
        static int instance_count;
    public:
        Counter() : value(0) {
            instance_count++;
        }
        ~Counter() {
            instance_count--;
        }
        int getValue() const {
            return value;
        }
        void increment() {
            value++;
        }
        static int getInstanceCount() {
            return instance_count;
        }
        static void resetInstanceCount() {
            instance_count = 0;
        }
    };

    int Counter::instance_count = 0;

    Counter* counter_new() {
        return new Counter();
    }

    void counter_delete(Counter* self) {
        delete self;
    }

    int counter_getInstanceCount() {
        return Counter::getInstanceCount();
    }

    void counter_resetInstanceCount() {
        Counter::resetInstanceCount();
    }
#line 48
 struct Counter_48;
#line 48
namespace hicc { template<> struct MethodsType<Counter, void> { typedef Counter_48 methods_type; }; }
#line 48
 struct Counter_48 {
#line 48
typedef Counter Self; typedef void SelfContainer; typedef Counter_48 SelfMethods;
#line 50
static void _hicc_test_50() { int (Self::* _50)() const = &Self::getValue; (void)_50; }
#line 50
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getValue));
#line 53
static void _hicc_test_53() { void (Self::* _53)() = &Self::increment; (void)_53; }
#line 53
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::increment));
#line 48
};
#line 59
EXPORT_METHODS_BEG(class_static) {
#line 63
static void _hicc_test_63() { Counter* (* _63)() = &counter_new; (void)_63; }
#line 63
EXPORT_METHOD_IN(void, ExportMethods, ((Counter* (*)())&counter_new));
#line 66
static void _hicc_test_66() { void (* _66)(Counter* self) = &counter_delete; (void)_66; }
#line 66
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Counter* self))&counter_delete));
#line 69
static void _hicc_test_69() { int (* _69)() = &counter_getInstanceCount; (void)_69; }
#line 69
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)())&counter_getInstanceCount));
#line 72
static void _hicc_test_72() { void (* _72)() = &counter_resetInstanceCount; (void)_72; }
#line 72
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)())&counter_resetInstanceCount));
#line 59
} EXPORT_METHODS_END();

