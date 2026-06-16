#include <hicc/hicc.hpp>
#line 0 R"(src/std_test/std_set.rs)"
#line 6
#include <hicc/std/string.hpp>
        #include <hicc/std/set.hpp>
#line 4
EXPORT_METHODS_BEG(hicc_std_set) {
#line 20
static void _hicc_test_20() { std::unique_ptr<std::set<int>> (* _20)() = &hicc::make_unique<std::set<int>>; (void)_20; }
#line 20
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::set<int>> (*)())&hicc::make_unique<std::set<int>>));
#line 23
static void _hicc_test_23() { std::unique_ptr<std::set<std::string>> (* _23)() = &hicc::make_unique<std::set<std::string>>; (void)_23; }
#line 23
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::set<std::string>> (*)())&hicc::make_unique<std::set<std::string>>));
#line 27
static void _hicc_test_27() { std::unique_ptr<std::multiset<int>> (* _27)() = &hicc::make_unique<std::multiset<int>>; (void)_27; }
#line 27
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::multiset<int>> (*)())&hicc::make_unique<std::multiset<int>>));
#line 30
static void _hicc_test_30() { std::unique_ptr<std::multiset<std::string>> (* _30)() = &hicc::make_unique<std::multiset<std::string>>; (void)_30; }
#line 30
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::multiset<std::string>> (*)())&hicc::make_unique<std::multiset<std::string>>));
#line 4
} EXPORT_METHODS_END();

