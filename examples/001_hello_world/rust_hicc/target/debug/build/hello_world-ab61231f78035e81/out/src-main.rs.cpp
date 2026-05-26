#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include "hello_world.h"
#line 6
EXPORT_METHODS_BEG(hello_world) {
#line 8
static void _hicc_test_8() { void (* _8)(void) = &hello_world; (void)_8; }
#line 8
static void _d47546274fd3a0f32509e8de2cd8ae12_8(){ hello_world(); }
#line 8
EXPORT_METHOD_IN(void, ExportMethods, _d47546274fd3a0f32509e8de2cd8ae12_8);
#line 6
} EXPORT_METHODS_END();

