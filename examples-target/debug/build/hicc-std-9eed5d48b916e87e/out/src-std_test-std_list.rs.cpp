#include <hicc/hicc.hpp>
#line 0 R"(src/std_test/std_list.rs)"
#line 6
#include <hicc/std/string.hpp>
        #include <hicc/std/list.hpp>
#line 4
EXPORT_METHODS_BEG(hicc_std_list) {
#line 13
static void _hicc_test_13() { std::unique_ptr<std::list<int>> (* _13)() = &hicc::make_unique<std::list<int>>; (void)_13; }
#line 13
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::list<int>> (*)())&hicc::make_unique<std::list<int>>));
#line 18
static void _hicc_test_18() { std::unique_ptr<std::list<std::string>> (* _18)() = &hicc::make_unique<std::list<std::string>>; (void)_18; }
#line 18
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::list<std::string>> (*)())&hicc::make_unique<std::list<std::string>>));
#line 4
} EXPORT_METHODS_END();

