#include <hicc/hicc.hpp>
#line 0 R"(src/std_test/std_map.rs)"
#line 6
#include <hicc/std/string.hpp>
        #include <hicc/std/map.hpp>
#line 4
EXPORT_METHODS_BEG(hicc_std_map) {
#line 21
static void _hicc_test_21() { std::map<std::string, std::string> (* _21)() = &hicc::make_constructor<std::map<std::string, std::string>>; (void)_21; }
#line 21
EXPORT_METHOD_IN(void, ExportMethods, ((std::map<std::string, std::string> (*)())&hicc::make_constructor<std::map<std::string, std::string>>));
#line 24
static void _hicc_test_24() { std::map<std::string, int> (* _24)() = &hicc::make_constructor<std::map<std::string, int>>; (void)_24; }
#line 24
EXPORT_METHOD_IN(void, ExportMethods, ((std::map<std::string, int> (*)())&hicc::make_constructor<std::map<std::string, int>>));
#line 27
static void _hicc_test_27() { std::map<int, std::string> (* _27)() = &hicc::make_constructor<std::map<int, std::string>>; (void)_27; }
#line 27
EXPORT_METHOD_IN(void, ExportMethods, ((std::map<int, std::string> (*)())&hicc::make_constructor<std::map<int, std::string>>));
#line 30
static void _hicc_test_30() { std::map<int, int> (* _30)() = &hicc::make_constructor<std::map<int, int>>; (void)_30; }
#line 30
EXPORT_METHOD_IN(void, ExportMethods, ((std::map<int, int> (*)())&hicc::make_constructor<std::map<int, int>>));
#line 43
static void _hicc_test_43() { std::multimap<std::string, std::string> (* _43)() = &hicc::make_constructor<std::multimap<std::string, std::string>>; (void)_43; }
#line 43
EXPORT_METHOD_IN(void, ExportMethods, ((std::multimap<std::string, std::string> (*)())&hicc::make_constructor<std::multimap<std::string, std::string>>));
#line 46
static void _hicc_test_46() { std::multimap<std::string, int> (* _46)() = &hicc::make_constructor<std::multimap<std::string, int>>; (void)_46; }
#line 46
EXPORT_METHOD_IN(void, ExportMethods, ((std::multimap<std::string, int> (*)())&hicc::make_constructor<std::multimap<std::string, int>>));
#line 49
static void _hicc_test_49() { std::multimap<int, std::string> (* _49)() = &hicc::make_constructor<std::multimap<int, std::string>>; (void)_49; }
#line 49
EXPORT_METHOD_IN(void, ExportMethods, ((std::multimap<int, std::string> (*)())&hicc::make_constructor<std::multimap<int, std::string>>));
#line 52
static void _hicc_test_52() { std::multimap<int, int> (* _52)() = &hicc::make_constructor<std::multimap<int, int>>; (void)_52; }
#line 52
EXPORT_METHOD_IN(void, ExportMethods, ((std::multimap<int, int> (*)())&hicc::make_constructor<std::multimap<int, int>>));
#line 4
} EXPORT_METHODS_END();

