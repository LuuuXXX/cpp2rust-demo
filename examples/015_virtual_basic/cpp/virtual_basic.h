#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct Shape;
struct Circle;

struct Shape* shape_new(void);
void shape_delete(struct Shape* self);

double shape_area(struct Shape* self);
const char* shape_getName(struct Shape* self);

struct Circle* circle_new(double radius);
void circle_delete(struct Circle* self);

double circle_area(struct Circle* self);
double circle_getRadius(struct Circle* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
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

#endif
