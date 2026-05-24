// Example 09: Namespaces
// C++ features: namespace declaration, nested namespaces, inline namespaces, using directives

#include <cmath>

// Basic namespace
namespace Math {
    const double PI = 3.14159265358979323846;

    int add(int a, int b) {
        return a + b;
    }

    int subtract(int a, int b) {
        return a - b;
    }

    double circle_area(double radius) {
        return PI * radius * radius;
    }

    namespace Advanced {
        double sin(double x) {
            return std::sin(x);
        }

        double cos(double x) {
            return std::cos(x);
        }
    }
}

// Nested namespace
namespace Outer {
    namespace Inner {
        int nested_value = 42;

        int nested_function(int x) {
            return x * 2;
        }
    }
}

// Inline namespace (for versioning)
namespace Version {
    inline namespace v1 {
        int version_function() {
            return 1;
        }
    }

    namespace v2 {
        int version_function() {
            return 2;
        }
    }
}

// Namespace alias
namespace MyMath = Math;
namespace OuterInner = Outer::Inner;

// Class inside namespace
namespace Shapes {
    class Point {
    private:
        double x, y;
    public:
        Point() : x(0), y(0) {}
        Point(double x_, double y_) : x(x_), y(y_) {}
        double get_x() const { return x; }
        double get_y() const { return y; }
    };

    class Circle {
    private:
        Point center;
        double radius;
    public:
        Circle(Point c, double r) : center(c), radius(r) {}
        double area() const {
            return Math::PI * radius * radius;
        }
    };
}
