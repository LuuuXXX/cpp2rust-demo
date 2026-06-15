#ifdef _MSC_VER
#define _USE_MATH_DEFINES
#endif
#include "virtual_pure.h"
#include <iostream>
#include <cmath>
#include <cstring>

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
