#pragma once

namespace function_overload_ns {

// 命名空间内自由函数（含不同参数类型/个数的重载族）：无需 extern "C"，
// 由 hicc import_lib! 以 ns::fn() 直出绑定。
int add_int(int a, int b);
double add_double(double a, double b);
const char* add_strings(const char* a, const char* b);
int sum3(int a, int b, int c);

} // namespace function_overload_ns
