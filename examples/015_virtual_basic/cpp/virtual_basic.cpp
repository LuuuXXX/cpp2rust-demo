#include "virtual_basic.h"
#include <iostream>
#ifdef _MSC_VER
#define _USE_MATH_DEFINES
#endif
#include <cmath>
#include <cstring>
#include <string>

// Shape class implementations
Shape::Shape(const char* n) : name(n) {}

Shape::~Shape() {}

double Shape::area() const {
    return 0.0;
}

const char* Shape::getName() const {
    return name.c_str();
}

// Circle class implementations
Circle::Circle(double r) : Shape("Circle"), radius(r) {}

Circle::~Circle() {}

double Circle::area() const {
    return M_PI * radius * radius;
}

double Circle::getRadius() const {
    return radius;
}

// FFI wrapper functions
struct Shape* shape_new(void) {
    return new Shape("Shape");
}

void shape_delete(struct Shape* self) {
    delete self;
}

double shape_area(struct Shape* self) {
    return self->area();
}

const char* shape_getName(struct Shape* self) {
    return self->getName();
}

struct Circle* circle_new(double radius) {
    return new Circle(radius);
}

void circle_delete(struct Circle* self) {
    delete self;
}

double circle_area(struct Circle* self) {
    return self->area();
}

double circle_getRadius(struct Circle* self) {
    return self->getRadius();
}
