#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <string>
    #include <iostream>
    #include <cmath>

    class Shape {
    protected:
        std::string name;
    public:
        Shape(const char* n);
        virtual ~Shape();
        virtual double area() const;
        const char* getName() const;
    };

    class Circle : public Shape {
        double radius;
    public:
        Circle(double r);
        ~Circle() override;
        double area() const override;
        double getRadius() const;
    };

    Shape::Shape(const char* n) : name(n) {}

    Shape::~Shape() {}

    double Shape::area() const {
        return 0.0;
    }

    const char* Shape::getName() const {
        return name.c_str();
    }

    Circle::Circle(double r) : Shape("Circle"), radius(r) {}

    Circle::~Circle() {}

    double Circle::area() const {
        return M_PI * radius * radius;
    }

    double Circle::getRadius() const {
        return radius;
    }

    Shape* shape_new() {
        return new Shape("Shape");
    }

    void shape_delete(Shape* self) {
        delete self;
    }

    Circle* circle_new(double radius) {
        return new Circle(radius);
    }

    void circle_delete(Circle* self) {
        delete self;
    }
#line 67
 struct Shape_67;
#line 67
namespace hicc { template<> struct MethodsType<Shape, void> { typedef Shape_67 methods_type; }; }
#line 67
 struct Shape_67 {
#line 67
typedef Shape Self; typedef void SelfContainer; typedef Shape_67 SelfMethods;
#line 69
static void _hicc_test_69() { double (Self::* _69)() const = &Self::area; (void)_69; }
#line 69
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((double (Self::*)() const)&Self::area));
#line 72
static void _hicc_test_72() { const char* (Self::* _72)() const = &Self::getName; (void)_72; }
#line 72
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::getName));
#line 67
};
#line 78
 struct Circle_78;
#line 78
namespace hicc { template<> struct MethodsType<Circle, void> { typedef Circle_78 methods_type; }; }
#line 78
 struct Circle_78 {
#line 78
typedef Circle Self; typedef void SelfContainer; typedef Circle_78 SelfMethods;
#line 80
static void _hicc_test_80() { double (Self::* _80)() const = &Self::area; (void)_80; }
#line 80
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((double (Self::*)() const)&Self::area));
#line 83
static void _hicc_test_83() { const char* (Self::* _83)() const = &Self::getName; (void)_83; }
#line 83
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::getName));
#line 86
static void _hicc_test_86() { double (Self::* _86)() const = &Self::getRadius; (void)_86; }
#line 86
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((double (Self::*)() const)&Self::getRadius));
#line 78
};
#line 92
EXPORT_METHODS_BEG(virtual_basic) {
#line 97
static void _hicc_test_97() { Shape* (* _97)() = &shape_new; (void)_97; }
#line 97
EXPORT_METHOD_IN(void, ExportMethods, ((Shape* (*)())&shape_new));
#line 100
static void _hicc_test_100() { void (* _100)(Shape* self) = &shape_delete; (void)_100; }
#line 100
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Shape* self))&shape_delete));
#line 103
static void _hicc_test_103() { Circle* (* _103)(double radius) = &circle_new; (void)_103; }
#line 103
EXPORT_METHOD_IN(void, ExportMethods, ((Circle* (*)(double radius))&circle_new));
#line 106
static void _hicc_test_106() { void (* _106)(Circle* self) = &circle_delete; (void)_106; }
#line 106
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(Circle* self))&circle_delete));
#line 92
} EXPORT_METHODS_END();

