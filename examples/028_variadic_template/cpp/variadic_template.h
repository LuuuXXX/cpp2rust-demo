#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// 可变参数模板示例 - Sum 函数
// 由于 FFI 无法直接处理可变参数，我们提供固定参数的版本

// 求和函数 - 固定参数版本
// 0 个参数
int sum_zero(void);

// 1 个参数
int sum_1(int a);

// 2 个参数
int sum_2(int a, int b);

// 3 个参数
int sum_3(int a, int b, int c);

// 4 个参数
int sum_4(int a, int b, int c, int d);

// 5 个参数
int sum_5(int a, int b, int c, int d, int e);

// double 版本
double sum_double_2(double a, double b);
double sum_double_3(double a, double b, double c);
double sum_double_4(double a, double b, double c, double d);

// 获取参数数量信息
const char* sum_getFormat(int count);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
class SumCalculator {
public:
    static int calculate_zero();
    static int calculate_1(int a);
    static int calculate_2(int a, int b);
    static int calculate_3(int a, int b, int c);
    static int calculate_4(int a, int b, int c, int d);
    static int calculate_5(int a, int b, int c, int d, int e);
    static double calculate_double_2(double a, double b);
    static double calculate_double_3(double a, double b, double c);
    static double calculate_double_4(double a, double b, double c, double d);
    static const char* get_format(int count);
};

#endif
