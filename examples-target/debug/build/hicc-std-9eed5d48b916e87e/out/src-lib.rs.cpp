#include <hicc/hicc.hpp>
#line 0 R"(src/lib.rs)"
#line 115
#include <hicc/std/string.hpp>
#line 112
EXPORT_METHODS_BEG(hicc_std_string) {
#line 120
static void _hicc_test_120() { std::unique_ptr<std::string> (* _120)() = &hicc::make_unique<std::string>; (void)_120; }
#line 120
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::string> (*)())&hicc::make_unique<std::string>));
#line 123
static void _hicc_test_123() { std::unique_ptr<std::string> (* _123)(const char* &&) = &hicc::make_unique<std::string, const char*>; (void)_123; }
#line 123
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::string> (*)(const char* &&))&hicc::make_unique<std::string, const char*>));
#line 126
static void _hicc_test_126() { std::unique_ptr<std::string> (* _126)(const char* &&, size_t&&) = &hicc::make_unique<std::string, const char*, size_t>; (void)_126; }
#line 126
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::string> (*)(const char* &&, size_t&&))&hicc::make_unique<std::string, const char*, size_t>));
#line 129
static void _hicc_test_129() { std::unique_ptr<std::u16string> (* _129)() = &hicc::make_unique<std::u16string>; (void)_129; }
#line 129
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::u16string> (*)())&hicc::make_unique<std::u16string>));
#line 132
static void _hicc_test_132() { std::unique_ptr<std::u16string> (* _132)(const char16_t* &&, size_t&&) = &hicc::make_unique<std::u16string, const char16_t*, size_t>; (void)_132; }
#line 132
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::u16string> (*)(const char16_t* &&, size_t&&))&hicc::make_unique<std::u16string, const char16_t*, size_t>));
#line 135
static void _hicc_test_135() { std::unique_ptr<std::u32string> (* _135)() = &hicc::make_unique<std::u32string>; (void)_135; }
#line 135
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::u32string> (*)())&hicc::make_unique<std::u32string>));
#line 138
static void _hicc_test_138() { std::unique_ptr<std::u32string> (* _138)(const char32_t* &&, size_t&&) = &hicc::make_unique<std::u32string, const char32_t*, size_t>; (void)_138; }
#line 138
EXPORT_METHOD_IN(void, ExportMethods, ((std::unique_ptr<std::u32string> (*)(const char32_t* &&, size_t&&))&hicc::make_unique<std::u32string, const char32_t*, size_t>));
#line 112
} EXPORT_METHODS_END();

