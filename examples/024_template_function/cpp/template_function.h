#pragma once

namespace template_function_ns {

// 函数模板：按指针交换两个对象。模板必须在使用点实例化，
// 每个实例化（do_swap<int>、do_swap<double> …）是一个独立的具体函数。
template <typename T>
void do_swap(T* a, T* b) {
    T t = *a;
    *a = *b;
    *b = t;
}

// 函数模板：返回两者中较大的值。
template <typename T>
T max_value(T a, T b) {
    return a > b ? a : b;
}

// 锚点：作为本单元可链接的非模板符号。
int template_function_anchor();

} // namespace template_function_ns
