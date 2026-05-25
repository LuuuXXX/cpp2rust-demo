#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// 可变参数函数
int sum(int count, ...);

// 固定参数 + 可变参数
int print_formatted(const char* format, ...);

// FFI wrapper functions (Rust cannot call variadic functions directly)
int sum_3(int a, int b, int c);
int sum_5(int a, int b, int c, int d, int e);

#ifdef __cplusplus
}
#endif
