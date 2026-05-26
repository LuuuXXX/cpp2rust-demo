#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 5
#include <cstdint>


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


    int safe_add(int a, int b) noexcept {
        return a + b;
    }


    const int MAX_SIZE = 100;

    int get_max_size() {
        return MAX_SIZE;
    }
#line 40
 struct Counter_40;
#line 40
namespace hicc { template<> struct MethodsType<Counter, void> { typedef Counter_40 methods_type; }; }
#line 40
 struct Counter_40 {
#line 40
typedef Counter Self; typedef void SelfContainer; typedef Counter_40 SelfMethods;
#line 42
static void _hicc_test_42() { int (Self::* _42)() const = &Self::get; (void)_42; }
#line 42
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get));
#line 45
static void _hicc_test_45() { void (Self::* _45)() = &Self::increment; (void)_45; }
#line 45
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::increment));
#line 48
static void _hicc_test_48() { void (Self::* _48)() = &Self::decrement; (void)_48; }
#line 48
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::decrement));
#line 40
};
#line 54
EXPORT_METHODS_BEG(summary) {
#line 58
static void _hicc_test_58() { Counter* (* _58)() = &counter_new; (void)_58; }
#line 58
EXPORT_METHOD_IN(void, ExportMethods, ((Counter* (*)())&counter_new));
#line 61
static void _hicc_test_61() { void (* _61)(Counter* self) = &counter_delete; (void)_61; }
#line 61
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Counter* self))&counter_delete));
#line 64
static void _hicc_test_64() { int (* _64)(int a, int b) noexcept = &safe_add; (void)_64; }
#line 64
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int a, int b) noexcept)&safe_add));
#line 67
static void _hicc_test_67() { int (* _67)() = &get_max_size; (void)_67; }
#line 67
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)())&get_max_size));
#line 54
} EXPORT_METHODS_END();

