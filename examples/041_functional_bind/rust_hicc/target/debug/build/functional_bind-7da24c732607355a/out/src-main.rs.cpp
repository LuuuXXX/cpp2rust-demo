#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <string>


    class Adder {
    public:
        int base_value;
        Adder(int base) : base_value(base) {}
        int add(int value) const { return base_value + value; }
    };


    class Multiplier {
    public:
        int factor;
        Multiplier(int f) : factor(f) {}
        int multiply(int value) const { return factor * value; }
    };


    class StringProcessor {
    public:
        std::string target;
        StringProcessor() : target() {}
        void set_target(const char* t) { target = t ? t : ""; }
        int count_char(char ch) const {
            int count = 0;
            for (char c : target) {
                if (c == ch) count++;
            }
            return count;
        }
    };

    Adder* adder_new(int base_value) { return new Adder(base_value); }
    void adder_delete(Adder* self) { delete self; }

    Multiplier* multiplier_new(int factor) { return new Multiplier(factor); }
    void multiplier_delete(Multiplier* self) { delete self; }

    StringProcessor* string_processor_new() { return new StringProcessor(); }
    void string_processor_delete(StringProcessor* self) { delete self; }
#line 46
 struct Adder_46;
#line 46
namespace hicc { template<> struct MethodsType<Adder, void> { typedef Adder_46 methods_type; }; }
#line 46
 struct Adder_46 {
#line 46
typedef Adder Self; typedef void SelfContainer; typedef Adder_46 SelfMethods;
#line 48
static void _hicc_test_48() { int (Self::* _48)(int) const = &Self::add; (void)_48; }
#line 48
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(int) const)&Self::add));
#line 46
};
#line 54
 struct Multiplier_54;
#line 54
namespace hicc { template<> struct MethodsType<Multiplier, void> { typedef Multiplier_54 methods_type; }; }
#line 54
 struct Multiplier_54 {
#line 54
typedef Multiplier Self; typedef void SelfContainer; typedef Multiplier_54 SelfMethods;
#line 56
static void _hicc_test_56() { int (Self::* _56)(int) const = &Self::multiply; (void)_56; }
#line 56
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(int) const)&Self::multiply));
#line 54
};
#line 62
 struct StringProcessor_62;
#line 62
namespace hicc { template<> struct MethodsType<StringProcessor, void> { typedef StringProcessor_62 methods_type; }; }
#line 62
 struct StringProcessor_62 {
#line 62
typedef StringProcessor Self; typedef void SelfContainer; typedef StringProcessor_62 SelfMethods;
#line 64
static void _hicc_test_64() { void (Self::* _64)(const char*) = &Self::set_target; (void)_64; }
#line 64
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const char*))&Self::set_target));
#line 67
static void _hicc_test_67() { int (Self::* _67)(char) const = &Self::count_char; (void)_67; }
#line 67
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(char) const)&Self::count_char));
#line 62
};
#line 73
EXPORT_METHODS_BEG(functional_bind) {
#line 79
static void _hicc_test_79() { Adder* (* _79)(int) = &adder_new; (void)_79; }
#line 79
EXPORT_METHOD_IN(void, ExportMethods, ((Adder* (*)(int))&adder_new));
#line 82
static void _hicc_test_82() { void (* _82)(Adder* self) = &adder_delete; (void)_82; }
#line 82
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Adder* self))&adder_delete));
#line 85
static void _hicc_test_85() { Multiplier* (* _85)(int) = &multiplier_new; (void)_85; }
#line 85
EXPORT_METHOD_IN(void, ExportMethods, ((Multiplier* (*)(int))&multiplier_new));
#line 88
static void _hicc_test_88() { void (* _88)(Multiplier* self) = &multiplier_delete; (void)_88; }
#line 88
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Multiplier* self))&multiplier_delete));
#line 91
static void _hicc_test_91() { StringProcessor* (* _91)() = &string_processor_new; (void)_91; }
#line 91
EXPORT_METHOD_IN(void, ExportMethods, ((StringProcessor* (*)())&string_processor_new));
#line 94
static void _hicc_test_94() { void (* _94)(StringProcessor* self) = &string_processor_delete; (void)_94; }
#line 94
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(StringProcessor* self))&string_processor_delete));
#line 73
} EXPORT_METHODS_END();

