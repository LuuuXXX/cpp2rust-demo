#include "virtual_pure.h"
#include <iostream>
#ifdef _MSC_VER
#define _USE_MATH_DEFINES
#endif
#include <cmath>
#include <cstring>

// Circle class implementations
Circle::Circle(double r) : radius(r) {}

Circle::~Circle() {
    std::cout << "Deleting Circle" << std::endl;
}

double Circle::area() const {
    return M_PI * radius * radius;
}

const char* Circle::getName() const {
    return "Circle";
}

// Rectangle class implementations
Rectangle::Rectangle(double w, double h) : width(w), height(h) {}

Rectangle::~Rectangle() {
    std::cout << "Deleting Rectangle" << std::endl;
}

double Rectangle::area() const {
    return width * height;
}

const char* Rectangle::getName() const {
    return "Rectangle";
}

// FFI wrapper functions
struct AbstractShape* abstract_shape_create_circle(double radius) {
    return new Circle(radius);
}

struct AbstractShape* abstract_shape_create_rectangle(double width, double height) {
    return new Rectangle(width, height);
}

void abstract_shape_delete(struct AbstractShape* self) {
    if (self) {
        std::cout << "Deleting " << self->getName() << std::endl;
        delete self;
    }
}

double abstract_shape_area(struct AbstractShape* self) {
    return self->area();
}

const char* abstract_shape_getName(struct AbstractShape* self) {
    return self->getName();
}
