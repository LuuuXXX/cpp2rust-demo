#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct Point;

// 构造函数变体
struct Point* point_new_xy(int x, int y);
struct Point* point_newPolar(double r, double theta);
void point_delete(struct Point* self);

// 成员函数
int point_getX(struct Point* self);
int point_getY(struct Point* self);
double point_getMagnitude(struct Point* self);
double point_getAngle(struct Point* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
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

#endif
