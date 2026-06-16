#include "virtual_basic.h"

namespace virtual_basic_ns {

Shape::Shape() = default;
Shape::~Shape() = default;
double Shape::area() const { return 0.0; }

Circle::Circle(double r) : radius_(r) {}
Circle::~Circle() = default;
double Circle::area() const { return 3.14159265358979323846 * radius_ * radius_; }
double Circle::radius() const { return radius_; }

int virtual_basic_anchor() { return 0; }

} // namespace virtual_basic_ns
