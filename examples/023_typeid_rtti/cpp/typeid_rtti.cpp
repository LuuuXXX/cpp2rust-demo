#include "typeid_rtti.h"
#include <cmath>

namespace typeid_rtti_ns {

Circle::Circle(double r) : radius_(r) {}
Circle::~Circle() = default;
double Circle::area() const { return 3.14159265358979 * radius_ * radius_; }
double Circle::radius() const { return radius_; }

Rectangle::Rectangle(double w, double h) : width_(w), height_(h) {}
Rectangle::~Rectangle() = default;
double Rectangle::area() const { return width_ * height_; }

Triangle::Triangle(double b, double h) : base_(b), height_(h) {}
Triangle::~Triangle() = default;
double Triangle::area() const { return 0.5 * base_ * height_; }

int typeid_rtti_anchor() { return 0; }

} // namespace typeid_rtti_ns
