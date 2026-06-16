#include <hicc/hicc.hpp>
#line 0 R"(src/std_test/std_unordered_set.rs)"
#line 6
#include <hicc/std/string.hpp>
        #include <hicc/std/unordered_set.hpp>
#line 4
EXPORT_METHODS_BEG(hicc_std_unordered_set) {
#line 21
static void _hicc_test_21() { std::unordered_set<int> (* _21)() = &hicc::make_constructor<std::unordered_set<int>>; (void)_21; }
#line 21
EXPORT_METHOD_IN(void, ExportMethods, ((std::unordered_set<int> (*)())&hicc::make_constructor<std::unordered_set<int>>));
#line 24
static void _hicc_test_24() { std::unordered_set<std::string> (* _24)() = &hicc::make_constructor<std::unordered_set<std::string>>; (void)_24; }
#line 24
EXPORT_METHOD_IN(void, ExportMethods, ((std::unordered_set<std::string> (*)())&hicc::make_constructor<std::unordered_set<std::string>>));
#line 28
static void _hicc_test_28() { std::unordered_multiset<int> (* _28)() = &hicc::make_constructor<std::unordered_multiset<int>>; (void)_28; }
#line 28
EXPORT_METHOD_IN(void, ExportMethods, ((std::unordered_multiset<int> (*)())&hicc::make_constructor<std::unordered_multiset<int>>));
#line 31
static void _hicc_test_31() { std::unordered_multiset<std::string> (* _31)() = &hicc::make_constructor<std::unordered_multiset<std::string>>; (void)_31; }
#line 31
EXPORT_METHOD_IN(void, ExportMethods, ((std::unordered_multiset<std::string> (*)())&hicc::make_constructor<std::unordered_multiset<std::string>>));
#line 4
} EXPORT_METHODS_END();

