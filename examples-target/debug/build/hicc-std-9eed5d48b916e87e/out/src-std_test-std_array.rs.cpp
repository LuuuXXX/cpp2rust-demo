#include <hicc/hicc.hpp>
#line 0 R"(src/std_test/std_array.rs)"
#line 6
#include <hicc/std/string.hpp>
        #include <hicc/std/array.hpp>
#line 4
EXPORT_METHODS_BEG(hicc_std_array) {
#line 14
static void _hicc_test_14() { std::unique_ptr<std::array<int, 10>> (* _14)() = &hicc::make_unique<std::array<int, 10>>; (void)_14; }
#line 14
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::array<int, 10>> (*)())&hicc::make_unique<std::array<int, 10>>));
#line 20
static void _hicc_test_20() { std::unique_ptr<std::array<std::string, 10>> (* _20)() = &hicc::make_unique<std::array<std::string, 10>>; (void)_20; }
#line 20
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::array<std::string, 10>> (*)())&hicc::make_unique<std::array<std::string, 10>>));
#line 4
} EXPORT_METHODS_END();

