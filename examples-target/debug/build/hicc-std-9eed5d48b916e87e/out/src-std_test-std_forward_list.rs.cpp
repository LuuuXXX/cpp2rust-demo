#include <hicc/hicc.hpp>
#line 0 R"(src/std_test/std_forward_list.rs)"
#line 6
#include <hicc/std/string.hpp>
        #include <hicc/std/forward_list.hpp>
#line 4
EXPORT_METHODS_BEG(hicc_std_forward_list) {
#line 13
static void _hicc_test_13() { std::unique_ptr<std::forward_list<int>> (* _13)() = &hicc::make_unique<std::forward_list<int>>; (void)_13; }
#line 13
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::forward_list<int>> (*)())&hicc::make_unique<std::forward_list<int>>));
#line 19
static void _hicc_test_19() { std::unique_ptr<std::forward_list<std::string>> (* _19)() = &hicc::make_unique<std::forward_list<std::string>>; (void)_19; }
#line 19
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::forward_list<std::string>> (*)())&hicc::make_unique<std::forward_list<std::string>>));
#line 4
} EXPORT_METHODS_END();

