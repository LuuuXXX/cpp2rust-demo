// Example 03: Basic Classes
// C++ features: class, constructor, destructor, access specifiers, methods, nested classes

#include <cmath>
#include <cstring>

// Simple Rectangle class
class Rectangle {
private:
    double width;
    double height;

public:
    // Default constructor
    Rectangle() : width(0), height(0) {}

    // Parameterized constructor
    Rectangle(double w, double h) : width(w), height(h) {}

    // Copy constructor
    Rectangle(const Rectangle& other) : width(other.width), height(other.height) {}

    // Destructor
    ~Rectangle() {}

    // Const method
    double area() const {
        return width * height;
    }

    // Const method
    double perimeter() const {
        return 2 * (width + height);
    }

    // Non-const method
    void resize(double w, double h) {
        width = w;
        height = h;
    }

    // Getter methods
    double get_width() const { return width; }
    double get_height() const { return height; }
};

// Point struct (similar to class but default public)
struct Point {
    double x;
    double y;
};

double point_distance(Point p1, Point p2) {
    double dx = p2.x - p1.x;
    double dy = p2.y - p1.y;
    return std::sqrt(dx * dx + dy * dy);
}

// Nested class
class Outer {
private:
    int outer_value;

public:
    Outer(int v) : outer_value(v) {}

    int get_outer_value() const { return outer_value; }

    class Inner {
    private:
        int inner_value;
    public:
        Inner(int v) : inner_value(v) {}
        int get_inner_value() const { return inner_value; }
        int get_outer_from_inner(Outer* o) const {
            return o ? o->outer_value : 0;
        }
    };

    Inner* create_inner(int v) const {
        return new Inner(v);
    }
};

// Class with static member
class Counter {
private:
    int count;
    static int total_count;

public:
    Counter() : count(0) { total_count++; }
    Counter(int c) : count(c) { total_count++; }

    void increment() { count++; }
    int get() const { return count; }
    static int get_total() { return total_count; }
};

int Counter::total_count = 0;
