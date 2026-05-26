#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>

    template<typename T>
    void do_swap(T* a, T* b) {
        T temp = *a;
        *a = *b;
        *b = temp;
    }

    void swap_int(int* a, int* b) {
        std::cout << "swap_int called" << std::endl;
        do_swap<int>(a, b);
    }

    void swap_double(double* a, double* b) {
        std::cout << "swap_double called" << std::endl;
        do_swap<double>(a, b);
    }

    void swap_char(char* a, char* b) {
        std::cout << "swap_char called" << std::endl;
        do_swap<char>(a, b);
    }

    void swap_int_array(int* arr, int i, int j) {
        std::cout << "swap_int_array: arr[" << i << "] <-> arr[" << j << "]" << std::endl;
        do_swap<int>(&arr[i], &arr[j]);
    }

    int get_int_array(int* arr, int idx) {
        return arr[idx];
    }

    void set_int_array(int* arr, int idx, int value) {
        arr[idx] = value;
    }
#line 41
EXPORT_METHODS_BEG(template_function) {
#line 43
static void _hicc_test_43() { void (* _43)(int* a, int* b) = &swap_int; (void)_43; }
#line 43
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(int* a, int* b))&swap_int));
#line 46
static void _hicc_test_46() { void (* _46)(double* a, double* b) = &swap_double; (void)_46; }
#line 46
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(double* a, double* b))&swap_double));
#line 49
static void _hicc_test_49() { void (* _49)(char* a, char* b) = &swap_char; (void)_49; }
#line 49
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(char* a, char* b))&swap_char));
#line 52
static void _hicc_test_52() { void (* _52)(int* arr, int i, int j) = &swap_int_array; (void)_52; }
#line 52
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(int* arr, int i, int j))&swap_int_array));
#line 55
static void _hicc_test_55() { int (* _55)(int* arr, int idx) = &get_int_array; (void)_55; }
#line 55
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(int* arr, int idx))&get_int_array));
#line 58
static void _hicc_test_58() { void (* _58)(int* arr, int idx, int value) = &set_int_array; (void)_58; }
#line 58
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(int* arr, int idx, int value))&set_int_array));
#line 41
} EXPORT_METHODS_END();

