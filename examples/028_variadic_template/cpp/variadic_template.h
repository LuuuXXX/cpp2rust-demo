#pragma once

namespace variadic_template_ns {

// 可变参数函数模板：用 C++17 折叠表达式对任意个数实参求和。
// 二元右折叠 `(args + ... + 0)` 对 0 个实参亦成立（结果为 0）。
template <typename... Args>
auto sum(Args... args) {
    return (args + ... + 0);
}

// 锚点：本单元可链接的非模板符号。
int variadic_template_anchor();

} // namespace variadic_template_ns
