#include "template_function.h"
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

void swap_char(unsigned char* a, unsigned char* b) {
    std::cout << "swap_char called" << std::endl;
    do_swap<unsigned char>(a, b);
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
