#include "class_constructor.h"
#include <iostream>
#include <cmath>

Point::Point(int x, int y) : x(x), y(y) {}
Point::~Point() {}
int Point::getX() const { return x; }
int Point::getY() const { return y; }
double Point::getMagnitude() const {
    return std::sqrt(x * x + y * y);
}
double Point::getAngle() const {
    return std::atan2(y, x);
}

Point point_new_polar(double r, double theta) {
    int x = static_cast<int>(r * std::cos(theta));
    int y = static_cast<int>(r * std::sin(theta));
    std::cout << "Point created: polar(" << r << ", " << theta << ") -> xy(" << x << ", " << y << ")" << std::endl;
    return Point(x, y);
}
