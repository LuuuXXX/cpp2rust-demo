#include <hicc/hicc.hpp>
#line 0 R"(src/lib.rs)"
#line 8
#include "class_basic.h"
    #include <hicc/std/string.hpp>
#line 15
 struct Counter_15;
#line 15
namespace hicc { template<> struct MethodsType<class_basic_ns::Counter, void> { typedef Counter_15 methods_type; }; }
#line 15
 struct Counter_15 {
#line 15
typedef class_basic_ns::Counter Self; typedef void SelfContainer; typedef Counter_15 SelfMethods;
#line 17
static void _hicc_test_17() { void (Self::* _17)() = &Self::inc; (void)_17; }
#line 17
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::inc));
#line 20
static void _hicc_test_20() { void (Self::* _20)(int) = &Self::inc_by; (void)_20; }
#line 20
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(int))&Self::inc_by));
#line 23
static void _hicc_test_23() { void (Self::* _23)() = &Self::reset; (void)_23; }
#line 23
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)())&Self::reset));
#line 26
static void _hicc_test_26() { int (Self::* _26)() const = &Self::count; (void)_26; }
#line 26
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::count));
#line 30
static void _hicc_test_30() { const std::string& (Self::* _30)() const = &Self::name; (void)_30; }
#line 30
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const std::string& (Self::*)() const)&Self::name));
#line 15
};
#line 43
EXPORT_METHODS_BEG(class_basic) {
#line 45
static void _hicc_test_45() { std::unique_ptr<class_basic_ns::Counter> (* _45)() = &hicc::make_unique<class_basic_ns::Counter>; (void)_45; }
#line 45
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<class_basic_ns::Counter> (*)())&hicc::make_unique<class_basic_ns::Counter>));
#line 48
static void _hicc_test_48() { std::unique_ptr<class_basic_ns::Counter> (* _48)(const std::string&) = &hicc::make_unique<class_basic_ns::Counter, const std::string&>; (void)_48; }
#line 48
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<class_basic_ns::Counter> (*)(const std::string&))&hicc::make_unique<class_basic_ns::Counter, const std::string&>));
#line 43
} EXPORT_METHODS_END();

