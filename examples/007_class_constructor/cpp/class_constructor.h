#pragma once

class Point {
    int x;
    int y;
public:
    Point(int x, int y);
    ~Point();
    int getX() const;
    int getY() const;
    double getMagnitude() const;
    double getAngle() const;
};

Point point_new_polar(double r, double theta);
