#pragma once

#ifdef __cplusplus
extern "C" {
#endif

void swap_int(int* a, int* b);
void swap_double(double* a, double* b);
void swap_char(char* a, char* b);
void swap_int_array(int* arr, int i, int j);

int get_int_array(int* arr, int idx);
void set_int_array(int* arr, int idx, int value);

#ifdef __cplusplus
}
#endif
