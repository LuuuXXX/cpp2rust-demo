#pragma once

namespace typeid_rtti_ns {

// 多态抽象基类（含虚函数 → 启用 RTTI）。无公有构造，不可实例化。
class Shape {
public:
    virtual ~Shape() = default;
    virtual double area() const = 0;
};

// 具体实现：圆
class Circle : public Shape {
public:
    explicit Circle(double r);
    ~Circle() override;
    double area() const override;
    double radius() const;
private:
    double radius_;
};

// 具体实现：矩形
class Rectangle : public Shape {
public:
    Rectangle(double w, double h);
    ~Rectangle() override;
    double area() const override;
private:
    double width_;
    double height_;
};

// 具体实现：三角形
class Triangle : public Shape {
public:
    Triangle(double b, double h);
    ~Triangle() override;
    double area() const override;
private:
    double base_;
    double height_;
};

// 锚点：触发 detect_idiomatic_mode 走直出路径。
int typeid_rtti_anchor();

} // namespace typeid_rtti_ns
