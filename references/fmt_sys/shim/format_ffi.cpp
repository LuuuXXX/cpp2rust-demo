// {fmt} extern-C 包装层（format_ffi.cpp）
//
// 封装 fmt::format / fmt::to_string 等核心 API 为 extern "C" 接口，
// 供 cpp2rust-demo 工具从中提取 hicc import_lib! FFI 绑定。
//
// 使用系统安装的 libfmt-dev（Ubuntu: apt-get install libfmt-dev）。
// 头文件路径：/usr/include/fmt/core.h

#include <fmt/core.h>
#include <fmt/format.h>
#include <cstring>
#include <cstdlib>

extern "C" {

/// 将整数格式化为 C 字符串（调用方负责用 fmt_free_string 释放）
char* fmt_format_int(int val) {
    std::string s = fmt::format("{}", val);
    char* buf = static_cast<char*>(std::malloc(s.size() + 1));
    if (buf) std::memcpy(buf, s.c_str(), s.size() + 1);
    return buf;
}

/// 将浮点数格式化为 C 字符串
char* fmt_format_double(double val) {
    std::string s = fmt::format("{}", val);
    char* buf = static_cast<char*>(std::malloc(s.size() + 1));
    if (buf) std::memcpy(buf, s.c_str(), s.size() + 1);
    return buf;
}

/// 使用 {} 占位符格式化两个整数
char* fmt_format_two_ints(int a, int b) {
    std::string s = fmt::format("{} {}", a, b);
    char* buf = static_cast<char*>(std::malloc(s.size() + 1));
    if (buf) std::memcpy(buf, s.c_str(), s.size() + 1);
    return buf;
}

/// 释放由 fmt_format_* 函数分配的字符串
void fmt_free_string(char* ptr) {
    std::free(ptr);
}

/// 返回格式化后字符串的长度（不含终止符）
size_t fmt_format_int_len(int val) {
    return fmt::format("{}", val).size();
}

} // extern "C"
