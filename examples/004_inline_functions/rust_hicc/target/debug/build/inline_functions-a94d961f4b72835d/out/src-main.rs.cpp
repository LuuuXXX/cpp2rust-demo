#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include "inline_functions.h"
#line 6
EXPORT_METHODS_BEG(inline_functions) {
#line 10
static void _hicc_test_10() { int (* _10)(int, int) = &min; (void)_10; }
#line 10
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int, int))&min));
#line 13
static void _hicc_test_13() { int (* _13)(int, int) = &max; (void)_13; }
#line 13
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int, int))&max));
#line 17
static void _hicc_test_17() { int (* _17)(int, int) = &min_v2; (void)_17; }
#line 17
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int, int))&min_v2));
#line 20
static void _hicc_test_20() { int (* _20)(int, int) = &max_v2; (void)_20; }
#line 20
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int, int))&max_v2));
#line 6
} EXPORT_METHODS_END();

