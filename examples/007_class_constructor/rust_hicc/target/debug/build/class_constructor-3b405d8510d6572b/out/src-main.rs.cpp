#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>
    #include <cmath>

    class Point {
        int x;
        int y;
    public:
        Point(int x, int y) : x(x), y(y) {}
        ~Point() {}
        int getX() const { return x; }
        int getY() const { return y; }
        double getMagnitude() const { return std::sqrt(x * x + y * y); }
        double getAngle() const { return std::atan2(y, x); }
    };

    Point* point_new_xy(int x, int y) {
        return new Point(x, y);
    }

    Point* point_new_polar(double r, double theta) {
        int x = static_cast<int>(r * std::cos(theta));
        int y = static_cast<int>(r * std::sin(theta));
        std::cout << "Point created: polar(" << r << ", " << theta << ") -> xy(" << x << ", " << y << ")" << std::endl;
        return new Point(x, y);
    }

    void point_delete(Point* self) {
        delete self;
    }
#line 34
 struct Point_34;
#line 34
namespace hicc { template<> struct MethodsType<Point, void> { typedef Point_34 methods_type; }; }
#line 34
 struct Point_34 {
#line 34
typedef Point Self; typedef void SelfContainer; typedef Point_34 SelfMethods;
#line 36
static void _hicc_test_36() { int (Self::* _36)() const = &Self::getX; (void)_36; }
#line 36
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getX));
#line 39
static void _hicc_test_39() { int (Self::* _39)() const = &Self::getY; (void)_39; }
#line 39
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getY));
#line 42
static void _hicc_test_42() { double (Self::* _42)() const = &Self::getMagnitude; (void)_42; }
#line 42
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((double (Self::*)() const)&Self::getMagnitude));
#line 45
static void _hicc_test_45() { double (Self::* _45)() const = &Self::getAngle; (void)_45; }
#line 45
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((double (Self::*)() const)&Self::getAngle));
#line 34
};
#line 51
EXPORT_METHODS_BEG(class_constructor) {
#line 55
static void _hicc_test_55() { Point* (* _55)(int x, int y) = &point_new_xy; (void)_55; }
#line 55
EXPORT_METHOD_IN(void, ExportMethods, ((Point* (*)(int x, int y))&point_new_xy));
#line 58
static void _hicc_test_58() { Point* (* _58)(double r, double theta) = &point_new_polar; (void)_58; }
#line 58
EXPORT_METHOD_IN(void, ExportMethods, ((Point* (*)(double r, double theta))&point_new_polar));
#line 61
static void _hicc_test_61() { void (* _61)(Point* self) = &point_delete; (void)_61; }
#line 61
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Point* self))&point_delete));
#line 51
} EXPORT_METHODS_END();

