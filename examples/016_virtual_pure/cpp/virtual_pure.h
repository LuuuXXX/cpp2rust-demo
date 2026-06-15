#pragma once

class AbstractShape {
public:
    virtual ~AbstractShape() = default;
    virtual double area() const = 0;
    virtual const char* getName() const = 0;
};

class Circle : public AbstractShape {
    double radius;
public:
    Circle(double r);
    ~Circle() override;
    double area() const override;
    const char* getName() const override;
};

class Rectangle : public AbstractShape {
    double width;
    double height;
public:
    Rectangle(double w, double h);
    ~Rectangle() override;
    double area() const override;
    const char* getName() const override;
};
