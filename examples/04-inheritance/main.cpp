// Example 04: Inheritance
// C++ features: single inheritance, multiple inheritance, access control, virtual keyword

#include <cmath>

// Base Shape class
class Shape {
protected:
    double x;
    double y;

public:
    Shape() : x(0), y(0) {}
    Shape(double x_, double y_) : x(x_), y(y_) {}

    virtual ~Shape() {}

    virtual double area() const = 0;
    virtual double perimeter() const = 0;

    void move(double dx, double dy) {
        x += dx;
        y += dy;
    }

    double get_x() const { return x; }
    double get_y() const { return y; }
};

// Rectangle - single inheritance
class Rectangle : public Shape {
private:
    double width;
    double height;

public:
    Rectangle() : Shape(), width(0), height(0) {}
    Rectangle(double x_, double y_, double w, double h)
        : Shape(x_, y_), width(w), height(h) {}

    double area() const override {
        return width * height;
    }

    double perimeter() const override {
        return 2 * (width + height);
    }

    double get_width() const { return width; }
    double get_height() const { return height; }
};

// Circle - single inheritance
class Circle : public Shape {
private:
    double radius;

public:
    Circle() : Shape(), radius(0) {}
    Circle(double x_, double y_, double r) : Shape(x_, y_), radius(r) {}

    double area() const override {
        return M_PI * radius * radius;
    }

    double perimeter() const override {
        return 2 * M_PI * radius;
    }

    double get_radius() const { return radius; }
};

// ColoredShape - multiple inheritance
class Color {
protected:
    int color;

public:
    Color() : color(0) {}
    Color(int c) : color(c) {}
    int get_color() const { return color; }
};

class ColoredRectangle : public Rectangle, public Color {
public:
    ColoredRectangle() : Rectangle(), Color(0) {}
    ColoredRectangle(double x_, double y_, double w, double h, int c)
        : Rectangle(x_, y_, w, h), Color(c) {}
};

// Diamond problem - virtual inheritance
class Animal {
public:
    virtual ~Animal() {}
    virtual void speak() const = 0;
};

class Mammal : virtual public Animal {
public:
    virtual void breathe() const {}
};

class Bird : virtual public Animal {
public:
    virtual void fly() const {}
};

class Bat : public Mammal, public Bird {
public:
    void speak() const override {
        // bats don't speak
    }
};
