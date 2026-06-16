#pragma once

#include <string>
#include <stdexcept>

namespace exception_basic_ns {

// Calculator：在方法边界内部捕获 C++ 异常并转换为错误码，避免异常跨 FFI 传播。
class Calculator {
    int last_error_;  // 0=none,1=invalid_argument,2=out_of_range,3=runtime_error
public:
    Calculator();

    int last_error() const;
    void clear_error();
    int has_error() const;
    int divide(int a, int b);
    int parse_int(const char* s);
};

// 锚点：本单元可链接的非模板符号。
int exception_basic_anchor();

} // namespace exception_basic_ns
