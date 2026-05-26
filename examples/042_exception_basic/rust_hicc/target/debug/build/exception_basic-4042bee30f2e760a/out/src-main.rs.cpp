#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <stdexcept>
    #include <cstring>


    struct ExceptionInfo {
        int code;
        char message[256];
        ExceptionInfo() : code(0) { message[0] = '\0'; }
        void clear() { code = 0; message[0] = '\0'; }
        void set(int c, const char* msg) {
            code = c;
            strncpy(message, msg, 255);
            message[255] = '\0';
        }
    };



    class Calculator {
        ExceptionInfo last_exception;
    public:
        Calculator() = default;
        ~Calculator() = default;
        void clear_exception() { last_exception.clear(); }
        int get_exception() { return last_exception.code; }

        int divide(int a, int b) {
            if (b == 0) {
                last_exception.set(3, "Division by zero");
                return 0;
            }
            return a / b;
        }

        int string_to_int(const char* str) {
            if (!str || *str == '\0') {
                last_exception.set(1, "Empty string");
                return 0;
            }
            char* end;
            int result = std::strtol(str, &end, 10);
            if (*end != '\0') {
                last_exception.set(1, "Invalid number format");
                return 0;
            }
            return result;
        }
    };


    Calculator* calculator_new() {
        return new Calculator();
    }

    void calculator_delete(Calculator* self) {
        delete self;
    }
#line 62
 struct Calculator_62;
#line 62
namespace hicc { template<> struct MethodsType<Calculator, void> { typedef Calculator_62 methods_type; }; }
#line 62
 struct Calculator_62 {
#line 62
typedef Calculator Self; typedef void SelfContainer; typedef Calculator_62 SelfMethods;
#line 64
static void _hicc_test_64() { void (Self::* _64)() = &Self::clear_exception; (void)_64; }
#line 64
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::clear_exception));
#line 67
static void _hicc_test_67() { int (Self::* _67)() = &Self::get_exception; (void)_67; }
#line 67
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)())&Self::get_exception));
#line 70
static void _hicc_test_70() { int (Self::* _70)(int a, int b) = &Self::divide; (void)_70; }
#line 70
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(int a, int b))&Self::divide));
#line 73
static void _hicc_test_73() { int (Self::* _73)(const char* str) = &Self::string_to_int; (void)_73; }
#line 73
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(const char* str))&Self::string_to_int));
#line 62
};
#line 79
EXPORT_METHODS_BEG(exception_basic) {
#line 83
static void _hicc_test_83() { Calculator* (* _83)() = &calculator_new; (void)_83; }
#line 83
EXPORT_METHOD_IN(void, ExportMethods, ((Calculator* (*)())&calculator_new));
#line 86
static void _hicc_test_86() { void (* _86)(Calculator* self) = &calculator_delete; (void)_86; }
#line 86
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Calculator* self))&calculator_delete));
#line 79
} EXPORT_METHODS_END();

