#pragma once

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
