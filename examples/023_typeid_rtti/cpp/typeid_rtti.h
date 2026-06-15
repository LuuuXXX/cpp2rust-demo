#pragma once

enum ShapeType {
    SHAPE_TYPE_CIRCLE = 0,
    SHAPE_TYPE_RECTANGLE = 1,
    SHAPE_TYPE_TRIANGLE = 2
};

class Shape {
public:
    virtual ~Shape() = default;
    virtual int getType() const = 0;
    virtual const char* getTypeName() const = 0;
    virtual double area() const = 0;
};

class Circle : public Shape {
    double radius;
public:
    Circle(double r);
    int getType() const override;
    const char* getTypeName() const override;
    double area() const override;
};

class Rectangle : public Shape {
    double width;
    double height;
public:
    Rectangle(double w, double h);
    int getType() const override;
    const char* getTypeName() const override;
    double area() const override;
};

class Triangle : public Shape {
    double base;
    double height;
public:
    Triangle(double b, double h);
    int getType() const override;
    const char* getTypeName() const override;
    double area() const override;
};
