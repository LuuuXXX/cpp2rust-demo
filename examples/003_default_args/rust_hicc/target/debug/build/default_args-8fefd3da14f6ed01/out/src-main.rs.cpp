#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include "default_args.h"
#line 6
EXPORT_METHODS_BEG(default_args) {
#line 9
static void _hicc_test_9() { int (* _9)(const char*, int) = &greet; (void)_9; }
#line 9
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(const char*, int))&greet));
#line 6
} EXPORT_METHODS_END();

