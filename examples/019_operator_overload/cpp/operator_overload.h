#pragma once

namespace operator_overload_ns {

// 运算符重载：算术、比较、一元、自增/自减、复合赋值
class Number {
public:
    explicit Number(int v);
    ~Number();

    int value() const;

    Number operator+(const Number& other) const;
    Number operator-(const Number& other) const;
    Number operator*(const Number& other) const;
    Number operator/(const Number& other) const;

    int compare(const Number& other) const;   // 普通方法（非运算符）

    Number operator-() const;                  // 一元负号
    Number& operator++();                      // 前置 ++
    Number& operator--();                      // 前置 --
    Number& operator+=(const Number& other);
    Number& operator-=(const Number& other);

private:
    int value_;
};

// 锚点：让 detect_idiomatic_mode 走直出路径。
int operator_overload_anchor();

} // namespace operator_overload_ns
