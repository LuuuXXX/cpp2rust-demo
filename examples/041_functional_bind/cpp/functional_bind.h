#pragma once

#include <functional>
#include <string>

namespace functional_bind_ns {

// Adder：内部持有由 std::bind 构造的 std::function，绑定 base 到加法左操作数。
// hicc 直出无需 extern-C 不透明指针 + *_delete，析构由 Rust Drop 自动完成。
class Adder {
    std::function<int(int)> add_;
public:
    explicit Adder(int base);
    Adder(const Adder&) = delete;
    Adder& operator=(const Adder&) = delete;

    int add(int value) const { return add_(value); }
};

// Multiplier：内部持有由 std::bind 构造的 std::function，绑定 factor 到乘法左操作数。
class Multiplier {
    std::function<int(int)> mul_;
public:
    explicit Multiplier(int factor);
    Multiplier(const Multiplier&) = delete;
    Multiplier& operator=(const Multiplier&) = delete;

    int multiply(int value) const { return mul_(value); }
};

// StringProcessor：持有目标字符串并统计指定字符出现次数。
class StringProcessor {
    std::string target_;
public:
    StringProcessor() = default;

    void set_target(const char* t) { target_ = t ? t : ""; }
    int count_char(char ch) const;
};

// 锚点：本单元可链接的非模板符号。
int functional_bind_anchor();

} // namespace functional_bind_ns
