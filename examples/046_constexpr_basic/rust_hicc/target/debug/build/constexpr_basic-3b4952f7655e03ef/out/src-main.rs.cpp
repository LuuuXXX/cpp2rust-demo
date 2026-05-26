#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <cstddef>


    const int ARRAY_SIZE = 10;

    namespace example {

        template<int N>
        constexpr int fibonacci() {
            if constexpr (N <= 1) {
                return N;
            } else {
                return fibonacci<N - 1>() + fibonacci<N - 2>();
            }
        }



        static constexpr int FIB_10 = fibonacci<10>();


        struct ConstexprPoint {
            int x;
            int y;

            constexpr ConstexprPoint(int x, int y) : x(x), y(y) {}

            constexpr int manhattan_distance() const {
                return (x > 0 ? x : -x) + (y > 0 ? y : -y);
            }
        };
    }


    int get_fibonacci_10() {
        return example::FIB_10;
    }

    int manhattan_distance(int x, int y) {
        const int dx = x > 0 ? x : -x;
        const int dy = y > 0 ? y : -y;
        return dx + dy;
    }

    int constexpr_sum_array(const int* arr, int size) {
        int sum = 0;
        for (int i = 0; i < size; ++i) {
            sum += arr[i];
        }
        return sum;
    }

    int constexpr_find_max(const int* arr, int size) {
        if (size <= 0) return 0;
        int max_val = arr[0];
        for (int i = 1; i < size; ++i) {
            if (arr[i] > max_val) {
                max_val = arr[i];
            }
        }
        return max_val;
    }

    int get_array_size() {
        return ARRAY_SIZE;
    }
#line 71
EXPORT_METHODS_BEG(constexpr_basic) {
#line 73
static void _hicc_test_73() { int (* _73)() = &get_fibonacci_10; (void)_73; }
#line 73
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)())&get_fibonacci_10));
#line 76
static void _hicc_test_76() { int (* _76)(int x, int y) = &manhattan_distance; (void)_76; }
#line 76
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int x, int y))&manhattan_distance));
#line 79
static void _hicc_test_79() { int (* _79)(const int* arr, int size) = &constexpr_sum_array; (void)_79; }
#line 79
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(const int* arr, int size))&constexpr_sum_array));
#line 82
static void _hicc_test_82() { int (* _82)(const int* arr, int size) = &constexpr_find_max; (void)_82; }
#line 82
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(const int* arr, int size))&constexpr_find_max));
#line 85
static void _hicc_test_85() { int (* _85)() = &get_array_size; (void)_85; }
#line 85
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)())&get_array_size));
#line 71
} EXPORT_METHODS_END();

