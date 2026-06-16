#include <hicc/hicc.hpp>
#line 0 R"(src/std_test/std_queue.rs)"
#line 6
#include <hicc/std/string.hpp>
        #include <hicc/std/queue.hpp>
#line 4
EXPORT_METHODS_BEG(hicc_std_queue) {
#line 13
static void _hicc_test_13() { std::unique_ptr<std::queue<int>> (* _13)() = &hicc::make_unique<std::queue<int>>; (void)_13; }
#line 13
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::queue<int>> (*)())&hicc::make_unique<std::queue<int>>));
#line 19
static void _hicc_test_19() { std::unique_ptr<std::queue<std::string>> (* _19)() = &hicc::make_unique<std::queue<std::string>>; (void)_19; }
#line 19
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::queue<std::string>> (*)())&hicc::make_unique<std::queue<std::string>>));
#line 25
static void _hicc_test_25() { std::unique_ptr<std::priority_queue<int>> (* _25)() = &hicc::make_unique<std::priority_queue<int>>; (void)_25; }
#line 25
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::priority_queue<int>> (*)())&hicc::make_unique<std::priority_queue<int>>));
#line 4
} EXPORT_METHODS_END();

