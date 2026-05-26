#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>
    #include <cmath>
    #include <cstring>

    class AbstractShape {
    public:
        virtual ~AbstractShape() = default;
        virtual double area() const = 0;
        virtual const char* getName() const = 0;
    };

    class Circle : public AbstractShape {
        double radius;
    public:
        Circle(double r);
        ~Circle() override;
        double area() const override;
        const char* getName() const override;
    };

    class Rectangle : public AbstractShape {
        double width;
        double height;
    public:
        Rectangle(double w, double h);
        ~Rectangle() override;
        double area() const override;
        const char* getName() const override;
    };

    Circle::Circle(double r) : radius(r) {}

    Circle::~Circle() {
        std::cout << "Deleting Circle" << std::endl;
    }

    double Circle::area() const {
        return M_PI * radius * radius;
    }

    const char* Circle::getName() const {
        return "Circle";
    }

    Rectangle::Rectangle(double w, double h) : width(w), height(h) {}

    Rectangle::~Rectangle() {
        std::cout << "Deleting Rectangle" << std::endl;
    }

    double Rectangle::area() const {
        return width * height;
    }

    const char* Rectangle::getName() const {
        return "Rectangle";
    }

    AbstractShape* abstract_shape_create_circle(double radius) {
        return new Circle(radius);
    }

    AbstractShape* abstract_shape_create_rectangle(double width, double height) {
        return new Rectangle(width, height);
    }

    void abstract_shape_delete(AbstractShape* self) {
        if (self) {
            std::cout << "Deleting " << self->getName() << std::endl;
            delete self;
        }
    }
#line 77
 struct AbstractShape_77;
#line 77
namespace hicc { template<> struct MethodsType<AbstractShape, void> { typedef AbstractShape_77 methods_type; }; }
#line 77
 struct AbstractShape_77 {
#line 77
typedef AbstractShape Self; typedef void SelfContainer; typedef AbstractShape_77 SelfMethods;
#line 79
static void _hicc_test_79() { double (Self::* _79)() const = &Self::area; (void)_79; }
#line 79
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((double (Self::*)() const)&Self::area));
#line 82
static void _hicc_test_82() { const char* (Self::* _82)() const = &Self::getName; (void)_82; }
#line 82
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((const char* (Self::*)() const)&Self::getName));
#line 77
};
#line 88
EXPORT_METHODS_BEG(virtual_pure) {
#line 92
static void _hicc_test_92() { AbstractShape* (* _92)(double radius) = &abstract_shape_create_circle; (void)_92; }
#line 92
EXPORT_METHOD_IN(void, ExportMethods, ((AbstractShape* (*)(double radius))&abstract_shape_create_circle));
#line 95
static void _hicc_test_95() { AbstractShape* (* _95)(double width, double height) = &abstract_shape_create_rectangle; (void)_95; }
#line 95
EXPORT_METHOD_IN(void, ExportMethods, ((AbstractShape* (*)(double width, double height))&abstract_shape_create_rectangle));
#line 98
static void _hicc_test_98() { void (* _98)(AbstractShape* self) = &abstract_shape_delete; (void)_98; }
#line 98
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(AbstractShape* self))&abstract_shape_delete));
#line 88
} EXPORT_METHODS_END();

