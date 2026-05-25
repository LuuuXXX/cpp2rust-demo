#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// 函数重载：不同的参数类型
int add_int(int a, int b);
double add_double(double a, double b);
const char* add_strings(const char* a, const char* b);

// 重载：不同的参数个数
int sum3(int a, int b, int c);

#ifdef __cplusplus
}
#endif
