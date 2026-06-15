#pragma once

#include <string>

class Shape {
protected:
    std::string name;
public:
    Shape(const char* n);
    virtual ~Shape();
    virtual double area() const;
    const char* getName() const;
};

class Circle : public Shape {
    double radius;
public:
    Circle(double r);
    ~Circle() override;
    double area() const override;
    double getRadius() const;
};
