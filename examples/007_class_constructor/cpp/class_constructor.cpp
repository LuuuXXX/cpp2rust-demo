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

struct Point* point_new_xy(int x, int y) {
    return new Point(x, y);
}

struct Point* point_newPolar(double r, double theta) {
    int x = static_cast<int>(r * std::cos(theta));
    int y = static_cast<int>(r * std::sin(theta));
    std::cout << "Point created: polar(" << r << ", " << theta << ") -> xy(" << x << ", " << y << ")" << std::endl;
    return new Point(x, y);
}

void point_delete(struct Point* self) {
    delete self;
}

int point_getX(struct Point* self) { return self->getX(); }
int point_getY(struct Point* self) { return self->getY(); }

double point_getMagnitude(struct Point* self) {
    return self->getMagnitude();
}

double point_getAngle(struct Point* self) {
    return self->getAngle();
}
