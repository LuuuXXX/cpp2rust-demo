#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <cstdio>




    class SumCalculator {
    public:
        static int calculate_zero() { return 0; }
        static int calculate_1(int a) { return a; }
        static int calculate_2(int a, int b) { return a + b; }
        static int calculate_3(int a, int b, int c) { return a + b + c; }
        static int calculate_4(int a, int b, int c, int d) { return a + b + c + d; }
        static int calculate_5(int a, int b, int c, int d, int e) { return a + b + c + d + e; }
        static double calculate_double_2(double a, double b) { return a + b; }
        static double calculate_double_3(double a, double b, double c) { return a + b + c; }
        static double calculate_double_4(double a, double b, double c, double d) { return a + b + c + d; }
        static const char* get_format(int count) {
            switch (count) {
                case 0: return "sum()";
                case 1: return "sum(%d)";
                case 2: return "sum(%d, %d)";
                case 3: return "sum(%d, %d, %d)";
                case 4: return "sum(%d, %d, %d, %d)";
                case 5: return "sum(%d, %d, %d, %d, %d)";
                default: return "unknown";
            }
        }
    };


    int sum_zero() {
        return SumCalculator::calculate_zero();
    }

    int sum_1(int a) {
        return SumCalculator::calculate_1(a);
    }

    int sum_2(int a, int b) {
        return SumCalculator::calculate_2(a, b);
    }

    int sum_3(int a, int b, int c) {
        return SumCalculator::calculate_3(a, b, c);
    }

    int sum_4(int a, int b, int c, int d) {
        return SumCalculator::calculate_4(a, b, c, d);
    }

    int sum_5(int a, int b, int c, int d, int e) {
        return SumCalculator::calculate_5(a, b, c, d, e);
    }

    double sum_double_2(double a, double b) {
        return SumCalculator::calculate_double_2(a, b);
    }

    double sum_double_3(double a, double b, double c) {
        return SumCalculator::calculate_double_3(a, b, c);
    }

    double sum_double_4(double a, double b, double c, double d) {
        return SumCalculator::calculate_double_4(a, b, c, d);
    }

    const char* sum_getFormat(int count) {
        return SumCalculator::get_format(count);
    }
#line 74
EXPORT_METHODS_BEG(variadic_template) {
#line 76
static void _hicc_test_76() { int (* _76)() = &sum_zero; (void)_76; }
#line 76
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)())&sum_zero));
#line 79
static void _hicc_test_79() { int (* _79)(int a) = &sum_1; (void)_79; }
#line 79
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int a))&sum_1));
#line 82
static void _hicc_test_82() { int (* _82)(int a, int b) = &sum_2; (void)_82; }
#line 82
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int a, int b))&sum_2));
#line 85
static void _hicc_test_85() { int (* _85)(int a, int b, int c) = &sum_3; (void)_85; }
#line 85
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int a, int b, int c))&sum_3));
#line 88
static void _hicc_test_88() { int (* _88)(int a, int b, int c, int d) = &sum_4; (void)_88; }
#line 88
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int a, int b, int c, int d))&sum_4));
#line 91
static void _hicc_test_91() { int (* _91)(int a, int b, int c, int d, int e) = &sum_5; (void)_91; }
#line 91
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int a, int b, int c, int d, int e))&sum_5));
#line 94
static void _hicc_test_94() { double (* _94)(double a, double b) = &sum_double_2; (void)_94; }
#line 94
EXPORT_METHOD_IN(void, ExportMethods, ((double (*)(double a, double b))&sum_double_2));
#line 97
static void _hicc_test_97() { double (* _97)(double a, double b, double c) = &sum_double_3; (void)_97; }
#line 97
EXPORT_METHOD_IN(void, ExportMethods, ((double (*)(double a, double b, double c))&sum_double_3));
#line 100
static void _hicc_test_100() { double (* _100)(double a, double b, double c, double d) = &sum_double_4; (void)_100; }
#line 100
EXPORT_METHOD_IN(void, ExportMethods, ((double (*)(double a, double b, double c, double d))&sum_double_4));
#line 103
static void _hicc_test_103() { const char* (* _103)(int count) = &sum_getFormat; (void)_103; }
#line 103
EXPORT_METHOD_IN(void, ExportMethods, ((const char* (*)(int count))&sum_getFormat));
#line 74
} EXPORT_METHODS_END();

