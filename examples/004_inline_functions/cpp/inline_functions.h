#pragma once

namespace inline_functions_ns {

// 命名空间内自由函数：无需 extern "C"，由 hicc import_lib! 以 ns::fn() 直出绑定。
// 说明：被 FFI 绑定的函数需在实现单元（.cpp）内定义，故此处统一声明、.cpp 定义。
int min(int a, int b);
int max(int a, int b);
int min_v2(int a, int b);
int max_v2(int a, int b);

} // namespace inline_functions_ns
