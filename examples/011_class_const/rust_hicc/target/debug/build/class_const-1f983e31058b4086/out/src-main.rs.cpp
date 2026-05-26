#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include "class_const.h"
#line 6
EXPORT_METHODS_BEG(class_const) {
#line 10
static void _hicc_test_10() { struct Calculator* (* _10)(void) = &calculator_new; (void)_10; }
#line 10
static struct Calculator* _57e4da21639e98a5e4c8076f8fcf527d_10(){ return calculator_new(); }
#line 10
EXPORT_METHOD_IN(void, ExportMethods, _57e4da21639e98a5e4c8076f8fcf527d_10);
#line 13
static void _hicc_test_13() { void (* _13)(struct Calculator*) = &calculator_delete; (void)_13; }
#line 13
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(struct Calculator*))&calculator_delete));
#line 17
static void _hicc_test_17() { int (* _17)(const struct Calculator*) = &calculator_getValue; (void)_17; }
#line 17
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(const struct Calculator*))&calculator_getValue));
#line 20
static void _hicc_test_20() { int (* _20)(const struct Calculator*) = &calculator_getHistoryCount; (void)_20; }
#line 20
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(const struct Calculator*))&calculator_getHistoryCount));
#line 24
static void _hicc_test_24() { void (* _24)(struct Calculator*, int) = &calculator_add; (void)_24; }
#line 24
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(struct Calculator*, int))&calculator_add));
#line 27
static void _hicc_test_27() { void (* _27)(struct Calculator*, int) = &calculator_subtract; (void)_27; }
#line 27
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(struct Calculator*, int))&calculator_subtract));
#line 30
static void _hicc_test_30() { void (* _30)(struct Calculator*) = &calculator_clear; (void)_30; }
#line 30
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(struct Calculator*))&calculator_clear));
#line 6
} EXPORT_METHODS_END();

