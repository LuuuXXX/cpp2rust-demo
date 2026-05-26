#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>


    class CallbackWrapper {
    public:
        int stored_value;
        int multiplier;
        CallbackWrapper(int mult) : stored_value(0), multiplier(mult) {}
        int process(int input) { stored_value = input * multiplier; return stored_value; }
        int get_value() const { return stored_value; }
    };


    class Processor {
    public:
        int stored_input;
        int stored_result;
        Processor() : stored_input(0), stored_result(0) {}
        void set_input(int input) { stored_input = input; }
        int get_input() const { return stored_input; }
        int get_result() const { return stored_result; }
    };

    CallbackWrapper* callback_wrapper_new(int multiplier) { return new CallbackWrapper(multiplier); }
    void callback_wrapper_delete(CallbackWrapper* self) { delete self; }

    Processor* processor_new() { return new Processor(); }
    void processor_delete(Processor* self) { delete self; }
#line 33
 struct CallbackWrapper_33;
#line 33
namespace hicc { template<> struct MethodsType<CallbackWrapper, void> { typedef CallbackWrapper_33 methods_type; }; }
#line 33
 struct CallbackWrapper_33 {
#line 33
typedef CallbackWrapper Self; typedef void SelfContainer; typedef CallbackWrapper_33 SelfMethods;
#line 35
static void _hicc_test_35() { int (Self::* _35)(int) = &Self::process; (void)_35; }
#line 35
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(int))&Self::process));
#line 38
static void _hicc_test_38() { int (Self::* _38)() const = &Self::get_value; (void)_38; }
#line 38
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get_value));
#line 33
};
#line 44
 struct Processor_44;
#line 44
namespace hicc { template<> struct MethodsType<Processor, void> { typedef Processor_44 methods_type; }; }
#line 44
 struct Processor_44 {
#line 44
typedef Processor Self; typedef void SelfContainer; typedef Processor_44 SelfMethods;
#line 46
static void _hicc_test_46() { void (Self::* _46)(int) = &Self::set_input; (void)_46; }
#line 46
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(int))&Self::set_input));
#line 49
static void _hicc_test_49() { int (Self::* _49)() const = &Self::get_input; (void)_49; }
#line 49
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get_input));
#line 52
static void _hicc_test_52() { int (Self::* _52)() const = &Self::get_result; (void)_52; }
#line 52
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::get_result));
#line 44
};
#line 58
EXPORT_METHODS_BEG(std_function) {
#line 63
static void _hicc_test_63() { CallbackWrapper* (* _63)(int) = &callback_wrapper_new; (void)_63; }
#line 63
EXPORT_METHOD_IN(void, ExportMethods, ((CallbackWrapper* (*)(int))&callback_wrapper_new));
#line 66
static void _hicc_test_66() { void (* _66)(CallbackWrapper* self) = &callback_wrapper_delete; (void)_66; }
#line 66
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(CallbackWrapper* self))&callback_wrapper_delete));
#line 69
static void _hicc_test_69() { Processor* (* _69)() = &processor_new; (void)_69; }
#line 69
EXPORT_METHOD_IN(void, ExportMethods, ((Processor* (*)())&processor_new));
#line 72
static void _hicc_test_72() { void (* _72)(Processor* self) = &processor_delete; (void)_72; }
#line 72
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Processor* self))&processor_delete));
#line 58
} EXPORT_METHODS_END();

