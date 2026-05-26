#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>
    #include <typeinfo>
    #include <cmath>

    enum ShapeType {
        SHAPE_TYPE_CIRCLE = 0,
        SHAPE_TYPE_RECTANGLE = 1,
        SHAPE_TYPE_TRIANGLE = 2
    };

    class Shape {
    public:
        ~Shape() = default;
        int getType() const { return -1; }
        const char* getTypeName() const { return "Shape"; }
        double area() const { return 0.0; }
    };

    class Circle : public Shape {
        double radius;
    public:
        Circle(double r);
        int getType() const;
        const char* getTypeName() const;
        double area() const;
    };

    class Rectangle : public Shape {
        double width;
        double height;
    public:
        Rectangle(double w, double h);
        int getType() const;
        const char* getTypeName() const;
        double area() const;
    };

    class Triangle : public Shape {
        double base;
        double height;
    public:
        Triangle(double b, double h);
        int getType() const;
        const char* getTypeName() const;
        double area() const;
    };

    Shape* shape_new_circle(double radius) {
        return new Circle(radius);
    }

    Shape* shape_new_rectangle(double width, double height) {
        return new Rectangle(width, height);
    }

    Shape* shape_new_triangle(double base, double height) {
        return new Triangle(base, height);
    }

    void shape_delete(Shape* self) {
        if (self) {
            std::cout << "Deleting " << self->getTypeName() << std::endl;
            delete self;
        }
    }

    int shape_getType(Shape* self) {
        return self->getType();
    }

    const char* shape_getTypeName(Shape* self) {
        return self->getTypeName();
    }

    double shape_area(Shape* self) {
        return self->area();
    }


    Circle::Circle(double r) : radius(r) {}
    int Circle::getType() const { return SHAPE_TYPE_CIRCLE; }
    const char* Circle::getTypeName() const { return "Circle"; }
    double Circle::area() const { return 3.14159 * radius * radius; }


    Rectangle::Rectangle(double w, double h) : width(w), height(h) {}
    int Rectangle::getType() const { return SHAPE_TYPE_RECTANGLE; }
    const char* Rectangle::getTypeName() const { return "Rectangle"; }
    double Rectangle::area() const { return width * height; }


    Triangle::Triangle(double b, double h) : base(b), height(h) {}
    int Triangle::getType() const { return SHAPE_TYPE_TRIANGLE; }
    const char* Triangle::getTypeName() const { return "Triangle"; }
    double Triangle::area() const { return 0.5 * base * height; }
#line 100
 struct Shape_100;
#line 100
namespace hicc { template<> struct MethodsType<Shape, void> { typedef Shape_100 methods_type; }; }
#line 100
 struct Shape_100 {
#line 100
typedef Shape Self; typedef void SelfContainer; typedef Shape_100 SelfMethods;
#line 102
static void _hicc_test_102() { int (Self::* _102)() const = &Self::getType; (void)_102; }
#line 102
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::getType));
#line 105
static void _hicc_test_105() { const char* (Self::* _105)() const = &Self::getTypeName; (void)_105; }
#line 105
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::getTypeName));
#line 108
static void _hicc_test_108() { double (Self::* _108)() const = &Self::area; (void)_108; }
#line 108
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((double (Self::*)() const)&Self::area));
#line 100
};
#line 114
EXPORT_METHODS_BEG(typeid_rtti) {
#line 118
static void _hicc_test_118() { Shape* (* _118)(double radius) = &shape_new_circle; (void)_118; }
#line 118
EXPORT_METHOD_IN(void, ExportMethods, ((Shape* (*)(double radius))&shape_new_circle));
#line 121
static void _hicc_test_121() { Shape* (* _121)(double width, double height) = &shape_new_rectangle; (void)_121; }
#line 121
EXPORT_METHOD_IN(void, ExportMethods, ((Shape* (*)(double width, double height))&shape_new_rectangle));
#line 124
static void _hicc_test_124() { Shape* (* _124)(double base, double height) = &shape_new_triangle; (void)_124; }
#line 124
EXPORT_METHOD_IN(void, ExportMethods, ((Shape* (*)(double base, double height))&shape_new_triangle));
#line 127
static void _hicc_test_127() { void (* _127)(Shape* self) = &shape_delete; (void)_127; }
#line 127
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Shape* self))&shape_delete));
#line 130
static void _hicc_test_130() { int (* _130)(Shape* self) = &shape_getType; (void)_130; }
#line 130
EXPORT_METHOD_IN(void, ExportMethods, ((int (*)(Shape* self))&shape_getType));
#line 133
static void _hicc_test_133() { const char* (* _133)(Shape* self) = &shape_getTypeName; (void)_133; }
#line 133
EXPORT_METHOD_IN(void, ExportMethods, ((const char* (*)(Shape* self))&shape_getTypeName));
#line 136
static void _hicc_test_136() { double (* _136)(Shape* self) = &shape_area; (void)_136; }
#line 136
EXPORT_METHOD_IN(void, ExportMethods, ((double (*)(Shape* self))&shape_area));
#line 114
} EXPORT_METHODS_END();

