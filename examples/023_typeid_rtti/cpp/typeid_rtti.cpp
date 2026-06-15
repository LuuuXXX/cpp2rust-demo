#include "typeid_rtti.h"
#include <cmath>

Circle::Circle(double r) : radius(r) {}
int Circle::getType() const { return SHAPE_TYPE_CIRCLE; }
const char* Circle::getTypeName() const { return "Circle"; }
double Circle::area() const { return 3.14159 * radius * radius; }

Rectangle::Rectangle(double w, double h) : width(w), height(h) {}
int Rectangle::getType() const { return SHAPE_TYPE_RECTANGLE; }
const char* Rectangle::getTypeName() const { return "Rectangle"; }
double Rectangle::area() const { return width * height; }

Triangle::Triangle(double b, double h) : base(b), height(h) {}
int Triangle::getType() const { return SHAPE_TYPE_TRIANGLE; }
const char* Triangle::getTypeName() const { return "Triangle"; }
double Triangle::area() const { return 0.5 * base * height; }
