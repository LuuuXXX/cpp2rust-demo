#include "virtual_pure.h"

namespace virtual_pure_ns {

Circle::Circle(double r) : radius_(r) {}
Circle::~Circle() = default;
double Circle::area() const { return 3.14159265358979323846 * radius_ * radius_; }
double Circle::radius() const { return radius_; }

Rectangle::Rectangle(double w, double h) : width_(w), height_(h) {}
Rectangle::~Rectangle() = default;
double Rectangle::area() const { return width_ * height_; }

int virtual_pure_anchor() { return 0; }

} // namespace virtual_pure_ns
