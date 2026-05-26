#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include "inheritance_single.h"
#line 6
EXPORT_METHODS_BEG(inheritance_single) {
#line 11
static void _hicc_test_11() { struct Animal* (* _11)(const char*) = &animal_new; (void)_11; }
#line 11
EXPORT_METHOD_IN(void, ExportMethods, ((struct Animal* (*)(const char*))&animal_new));
#line 14
static void _hicc_test_14() { void (* _14)(struct Animal*) = &animal_delete; (void)_14; }
#line 14
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(struct Animal*))&animal_delete));
#line 17
static void _hicc_test_17() { const char* (* _17)(struct Animal*) = &animal_getName; (void)_17; }
#line 17
EXPORT_METHOD_IN(void, ExportMethods, ((const char* (*)(struct Animal*))&animal_getName));
#line 20
static void _hicc_test_20() { void (* _20)(struct Animal*) = &animal_speak; (void)_20; }
#line 20
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(struct Animal*))&animal_speak));
#line 26
static void _hicc_test_26() { struct Dog* (* _26)(const char*) = &dog_new; (void)_26; }
#line 26
EXPORT_METHOD_IN(void, ExportMethods, ((struct Dog* (*)(const char*))&dog_new));
#line 29
static void _hicc_test_29() { void (* _29)(struct Dog*) = &dog_delete; (void)_29; }
#line 29
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(struct Dog*))&dog_delete));
#line 32
static void _hicc_test_32() { void (* _32)(struct Dog*) = &dog_bark; (void)_32; }
#line 32
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(struct Dog*))&dog_bark));
#line 36
static void _hicc_test_36() { const char* (* _36)(struct Dog*) = &dog_getName; (void)_36; }
#line 36
EXPORT_METHOD_IN(void, ExportMethods, ((const char* (*)(struct Dog*))&dog_getName));
#line 39
static void _hicc_test_39() { void (* _39)(struct Dog*) = &dog_speak; (void)_39; }
#line 39
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(struct Dog*))&dog_speak));
#line 6
} EXPORT_METHODS_END();

