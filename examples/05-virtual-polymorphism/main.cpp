// Example 05: Virtual Functions and Polymorphism
// C++ features: virtual functions, pure virtual, override, dynamic dispatch

#include <cmath>
#include <iostream>

class Shape {
public:
    virtual ~Shape() {}
    virtual double area() const = 0;
    virtual double perimeter() const = 0;
    virtual void describe() const {
        std::cout << "Shape" << std::endl;
    }
};

class Rectangle : public Shape {
private:
    double width;
    double height;
public:
    Rectangle(double w, double h) : width(w), height(h) {}
    double area() const override { return width * height; }
    double perimeter() const override { return 2 * (width + height); }
    void describe() const override {
        std::cout << "Rectangle: " << width << "x" << height << std::endl;
    }
};

class Circle : public Shape {
private:
    double radius;
public:
    Circle(double r) : radius(r) {}
    double area() const override { return M_PI * radius * radius; }
    double perimeter() const override { return 2 * M_PI * radius; }
    void describe() const override {
        std::cout << "Circle: r=" << radius << std::endl;
    }
};

class Triangle : public Shape {
private:
    double a, b, c;
public:
    Triangle(double a_, double b_, double c_) : a(a_), b(b_), c(c_) {}
    double area() const override {
        double s = (a + b + c) / 2;
        return std::sqrt(s * (s - a) * (s - b) * (s - c));
    }
    double perimeter() const override { return a + b + c; }
    void describe() const override {
        std::cout << "Triangle: sides=" << a << "," << b << "," << c << std::endl;
    }
};

// Factory pattern using virtual constructor
class ShapeFactory {
public:
    virtual ~ShapeFactory() {}
    virtual Shape* create() const = 0;
};

class RectangleFactory : public ShapeFactory {
public:
    Shape* create() const override {
        return new Rectangle(4.0, 3.0);
    }
};

class CircleFactory : public ShapeFactory {
public:
    Shape* create() const override {
        return new Circle(5.0);
    }
};

// Polymorphic function
double total_area(const Shape** shapes, int n) {
    double total = 0;
    for (int i = 0; i < n; i++) {
        if (shapes[i]) {
            total += shapes[i]->area();
        }
    }
    return total;
}
