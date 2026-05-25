#include "typeid_rtti.h"
#include <iostream>
#include <typeinfo>
#include <cmath>

struct Shape* shape_new_circle(double radius) {
    return new Circle(radius);
}

struct Shape* shape_new_rectangle(double width, double height) {
    return new Rectangle(width, height);
}

struct Shape* shape_new_triangle(double base, double height) {
    return new Triangle(base, height);
}

void shape_delete(struct Shape* self) {
    if (self) {
        std::cout << "Deleting " << self->getTypeName() << std::endl;
        delete self;
    }
}

int shape_getType(struct Shape* self) {
    return self->getType();
}

const char* shape_getTypeName(struct Shape* self) {
    return self->getTypeName();
}

double shape_area(struct Shape* self) {
    return self->area();
}

// Circle implementation
Circle::Circle(double r) : radius(r) {}
int Circle::getType() const { return SHAPE_TYPE_CIRCLE; }
const char* Circle::getTypeName() const { return "Circle"; }
double Circle::area() const { return 3.14159 * radius * radius; }

// Rectangle implementation
Rectangle::Rectangle(double w, double h) : width(w), height(h) {}
int Rectangle::getType() const { return SHAPE_TYPE_RECTANGLE; }
const char* Rectangle::getTypeName() const { return "Rectangle"; }
double Rectangle::area() const { return width * height; }

// Triangle implementation
Triangle::Triangle(double b, double h) : base(b), height(h) {}
int Triangle::getType() const { return SHAPE_TYPE_TRIANGLE; }
const char* Triangle::getTypeName() const { return "Triangle"; }
double Triangle::area() const { return 0.5 * base * height; }
