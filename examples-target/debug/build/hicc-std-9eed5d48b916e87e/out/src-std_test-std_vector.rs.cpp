#include <hicc/hicc.hpp>
#line 0 R"(src/std_test/std_vector.rs)"
#line 6
#include <hicc/std/string.hpp>
        #include <hicc/std/vector.hpp>
#line 4
EXPORT_METHODS_BEG(hicc_std_vector) {
#line 13
static void _hicc_test_13() { std::unique_ptr<std::vector<int>> (* _13)() = &hicc::make_unique<std::vector<int>>; (void)_13; }
#line 13
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::vector<int>> (*)())&hicc::make_unique<std::vector<int>>));
#line 19
static void _hicc_test_19() { std::unique_ptr<std::vector<std::string>> (* _19)() = &hicc::make_unique<std::vector<std::string>>; (void)_19; }
#line 19
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::vector<std::string>> (*)())&hicc::make_unique<std::vector<std::string>>));
#line 25
static void _hicc_test_25() { std::unique_ptr<std::vector<std::vector<std::string>>> (* _25)() = &hicc::make_unique<std::vector<std::vector<std::string>>>; (void)_25; }
#line 25
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::vector<std::vector<std::string>>> (*)())&hicc::make_unique<std::vector<std::vector<std::string>>>));
#line 29
static void _hicc_test_29() { std::unique_ptr<std::vector<bool>> (* _29)() = &hicc::make_unique<std::vector<bool>>; (void)_29; }
#line 29
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::vector<bool>> (*)())&hicc::make_unique<std::vector<bool>>));
#line 4
} EXPORT_METHODS_END();

