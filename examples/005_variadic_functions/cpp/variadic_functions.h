#pragma once

namespace variadic_functions_ns {

// 真正的 C 可变参数函数（Rust 无法直接经 FFI 调用，工具会跳过其绑定）。
int sum(int count, ...);
int print_formatted(const char* format, ...);

// FFI 固定参数包装函数：无需 extern "C"，由 hicc import_lib! 以 ns::fn() 直出绑定。
int sum_3(int a, int b, int c);
int sum_5(int a, int b, int c, int d, int e);

} // namespace variadic_functions_ns
