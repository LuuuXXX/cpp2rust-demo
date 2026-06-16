#include <hicc/hicc.hpp>
#line 0 R"(src/std_test/std_deque.rs)"
#line 6
#include <hicc/std/string.hpp>
        #include <hicc/std/deque.hpp>
#line 4
EXPORT_METHODS_BEG(hicc_std_deque) {
#line 13
static void _hicc_test_13() { std::unique_ptr<std::deque<int>> (* _13)() = &hicc::make_unique<std::deque<int>>; (void)_13; }
#line 13
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::deque<int>> (*)())&hicc::make_unique<std::deque<int>>));
#line 18
static void _hicc_test_18() { std::unique_ptr<std::deque<std::string>> (* _18)() = &hicc::make_unique<std::deque<std::string>>; (void)_18; }
#line 18
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::deque<std::string>> (*)())&hicc::make_unique<std::deque<std::string>>));
#line 4
} EXPORT_METHODS_END();

