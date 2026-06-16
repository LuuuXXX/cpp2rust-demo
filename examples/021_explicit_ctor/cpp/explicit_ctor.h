#pragma once

namespace explicit_ctor_ns {

// 显式构造函数：Widget(int) 允许隐式转换，explicit Widget(double) 禁止隐式转换。
class Widget {
public:
    Widget(int v);             // 非 explicit：int 可隐式转换为 Widget
    explicit Widget(double v); // explicit：double 必须显式构造

    ~Widget();

    int getValue() const;

private:
    int value_;
};

// 锚点：触发 detect_idiomatic_mode 走直出路径。
int explicit_ctor_anchor();

} // namespace explicit_ctor_ns
