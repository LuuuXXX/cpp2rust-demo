#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct Shape;

enum ShapeType {
    SHAPE_TYPE_CIRCLE = 0,
    SHAPE_TYPE_RECTANGLE = 1,
    SHAPE_TYPE_TRIANGLE = 2
};

struct Shape* shape_new_circle(double radius);
struct Shape* shape_new_rectangle(double width, double height);
struct Shape* shape_new_triangle(double base, double height);

void shape_delete(struct Shape* self);

int shape_getType(struct Shape* self);
const char* shape_getTypeName(struct Shape* self);
double shape_area(struct Shape* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
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

#endif
