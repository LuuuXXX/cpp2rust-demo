#pragma once

namespace virtual_pure_ns {

// 纯虚接口（抽象基类）：无法直接实例化
class AbstractShape {
public:
    virtual ~AbstractShape() = default;
    virtual double area() const = 0;   // 纯虚函数
};

// 具体实现 1
class Circle : public AbstractShape {
public:
    explicit Circle(double r);
    ~Circle() override;
    double area() const override;
    double radius() const;
private:
    double radius_;
};

// 具体实现 2
class Rectangle : public AbstractShape {
public:
    Rectangle(double w, double h);
    ~Rectangle() override;
    double area() const override;
private:
    double width_;
    double height_;
};

// 锚点：让 detect_idiomatic_mode 走直出路径。
int virtual_pure_anchor();

} // namespace virtual_pure_ns
