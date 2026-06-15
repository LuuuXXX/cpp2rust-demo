#ifdef _MSC_VER
#define _USE_MATH_DEFINES
#endif
#include "virtual_basic.h"
#include <iostream>
#include <cmath>
#include <cstring>
#include <string>

Shape::Shape(const char* n) : name(n) {}

Shape::~Shape() {}

double Shape::area() const {
    return 0.0;
}

const char* Shape::getName() const {
    return name.c_str();
}

Circle::Circle(double r) : Shape("Circle"), radius(r) {}

Circle::~Circle() {}

double Circle::area() const {
    return M_PI * radius * radius;
}

double Circle::getRadius() const {
    return radius;
}
