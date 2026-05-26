#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>

    class Counter {
        int value = 0;
    public:
        Counter() = default;
        ~Counter() = default;
        int get() const { return value; }
        void increment() { value++; }
        void decrement() { value--; }
    };

    Counter* counter_new() {
        return new Counter();
    }

    void counter_delete(Counter* self) {
        delete self;
    }
#line 24
 struct Counter_24;
#line 24
namespace hicc { template<> struct MethodsType<Counter, void> { typedef Counter_24 methods_type; }; }
#line 24
 struct Counter_24 {
#line 24
typedef Counter Self; typedef void SelfContainer; typedef Counter_24 SelfMethods;
#line 26
static void _hicc_test_26() { int (Self::* _26)() const = &Self::get; (void)_26; }
#line 26
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get));
#line 29
static void _hicc_test_29() { void (Self::* _29)() = &Self::increment; (void)_29; }
#line 29
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::increment));
#line 32
static void _hicc_test_32() { void (Self::* _32)() = &Self::decrement; (void)_32; }
#line 32
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::decrement));
#line 24
};
#line 38
EXPORT_METHODS_BEG(class_basic) {
#line 42
static void _hicc_test_42() { Counter* (* _42)() = &counter_new; (void)_42; }
#line 42
EXPORT_METHOD_IN(void, ExportMethods, ((Counter* (*)())&counter_new));
#line 45
static void _hicc_test_45() { void (* _45)(Counter*) = &counter_delete; (void)_45; }
#line 45
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Counter*))&counter_delete));
#line 38
} EXPORT_METHODS_END();

