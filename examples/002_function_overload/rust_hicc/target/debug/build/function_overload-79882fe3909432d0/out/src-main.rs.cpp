#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 4
#include "function_overload.h"
#line 8
EXPORT_METHODS_BEG(function_overload) {
#line 10
static void _hicc_test_10() { int (* _10)(int, int) = &add_int; (void)_10; }
#line 10
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int, int))&add_int));
#line 13
static void _hicc_test_13() { double (* _13)(double, double) = &add_double; (void)_13; }
#line 13
EXPORT_METHOD_IN(void, ExportMethods, ((double (*)(double, double))&add_double));
#line 16
static void _hicc_test_16() { const char* (* _16)(const char*, const char*) = &add_strings; (void)_16; }
#line 16
EXPORT_METHOD_IN(void, ExportMethods, ((const char* (*)(const char*, const char*))&add_strings));
#line 19
static void _hicc_test_19() { int (* _19)(int, int, int) = &sum3; (void)_19; }
#line 19
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int, int, int))&sum3));
#line 8
} EXPORT_METHODS_END();

