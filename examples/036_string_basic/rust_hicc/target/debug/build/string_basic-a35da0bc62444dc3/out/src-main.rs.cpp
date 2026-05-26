#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <string>
    #include <algorithm>
    #include <cctype>

    class String {
    public:
        std::string data;
        String() : data() {}
        explicit String(const char* str) : data(str ? str : "") {}
        explicit String(const char* str, unsigned long len) : data(str ? std::string(str, len) : "") {}
        ~String() { data.clear(); }
        unsigned long size() const { return data.size(); }
        unsigned long length() const { return data.length(); }
        bool empty() const { return data.empty(); }
        const char* c_str() const { return data.c_str(); }
        int compare(const char* other) const { return other ? data.compare(other) : 1; }
        bool equals(const char* other) const { return other ? data == other : data.empty(); }
        void append(const char* other) { if (other) data += other; }
        void clear() { data.clear(); }
        void to_upper() {
            std::transform(data.begin(), data.end(), data.begin(), ::toupper);
        }
        void to_lower() {
            std::transform(data.begin(), data.end(), data.begin(), ::tolower);
        }
    };

    String* string_new() { return new String(); }
    String* string_new_from(const char* str) { return new String(str); }
    void string_delete(String* self) { delete self; }
#line 35
 struct String_35;
#line 35
namespace hicc { template<> struct MethodsType<String, void> { typedef String_35 methods_type; }; }
#line 35
 struct String_35 {
#line 35
typedef String Self; typedef void SelfContainer; typedef String_35 SelfMethods;
#line 37
static void _hicc_test_37() { unsigned long (Self::* _37)() const = &Self::size; (void)_37; }
#line 37
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((unsigned long (Self::*)() const)&Self::size));
#line 40
static void _hicc_test_40() { unsigned long (Self::* _40)() const = &Self::length; (void)_40; }
#line 40
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((unsigned long (Self::*)() const)&Self::length));
#line 43
static void _hicc_test_43() { bool (Self::* _43)() const = &Self::empty; (void)_43; }
#line 43
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)() const)&Self::empty));
#line 46
static void _hicc_test_46() { const char* (Self::* _46)() const = &Self::c_str; (void)_46; }
#line 46
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::c_str));
#line 49
static void _hicc_test_49() { int (Self::* _49)(const char*) const = &Self::compare; (void)_49; }
#line 49
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(const char*) const)&Self::compare));
#line 52
static void _hicc_test_52() { bool (Self::* _52)(const char*) const = &Self::equals; (void)_52; }
#line 52
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((bool (Self::*)(const char*) const)&Self::equals));
#line 55
static void _hicc_test_55() { void (Self::* _55)(const char*) = &Self::append; (void)_55; }
#line 55
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(const char*))&Self::append));
#line 58
static void _hicc_test_58() { void (Self::* _58)() = &Self::clear; (void)_58; }
#line 58
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear));
#line 61
static void _hicc_test_61() { void (Self::* _61)() = &Self::to_upper; (void)_61; }
#line 61
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::to_upper));
#line 64
static void _hicc_test_64() { void (Self::* _64)() = &Self::to_lower; (void)_64; }
#line 64
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::to_lower));
#line 35
};
#line 70
EXPORT_METHODS_BEG(string_basic) {
#line 74
static void _hicc_test_74() { String* (* _74)() = &string_new; (void)_74; }
#line 74
EXPORT_METHOD_IN(void, ExportMethods, ((String* (*)())&string_new));
#line 77
static void _hicc_test_77() { String* (* _77)(const char* str) = &string_new_from; (void)_77; }
#line 77
EXPORT_METHOD_IN(void, ExportMethods, ((String* (*)(const char* str))&string_new_from));
#line 80
static void _hicc_test_80() { void (* _80)(String* self) = &string_delete; (void)_80; }
#line 80
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(String* self))&string_delete));
#line 70
} EXPORT_METHODS_END();

