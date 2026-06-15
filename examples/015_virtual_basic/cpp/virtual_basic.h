#pragma once

namespace virtual_basic_ns {

// 基类：含虚函数 area()
class Shape {
public:
    Shape();
    virtual ~Shape();
    virtual double area() const;   // 虚函数，基类默认返回 0
};

// 派生类：覆写虚函数 area()
class Circle : public Shape {
public:
    explicit Circle(double r);
    ~Circle() override;
    double area() const override;  // 覆写：π·r²
    double radius() const;
private:
    double radius_;
};

// 锚点：让 detect_idiomatic_mode 走直出路径。
int virtual_basic_anchor();

} // namespace virtual_basic_ns
