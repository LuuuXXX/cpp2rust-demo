// Example 17: Friend Functions and Classes
// C++ features: friend functions, friend classes, operator overloading with friends

#include <cstdint>

class Vector2D {
private:
    double x;
    double y;

public:
    Vector2D() : x(0), y(0) {}
    Vector2D(double x_, double y_) : x(x_), y(y_) {}

    double get_x() const { return x; }
    double get_y() const { return y; }

    // Friend function declaration
    friend Vector2D operator+(const Vector2D& a, const Vector2D& b);
    friend Vector2D operator-(const Vector2D& a, const Vector2D& b);
    friend Vector2D operator*(const Vector2D& v, double scalar);
    friend Vector2D operator*(double scalar, const Vector2D& v);
    friend bool operator==(const Vector2D& a, const Vector2D& b);
    friend double dot(const Vector2D& a, const Vector2D& b);
    friend class Vector2DFactory;
};

// Friend function definitions
Vector2D operator+(const Vector2D& a, const Vector2D& b) {
    return Vector2D(a.x + b.x, a.y + b.y);
}

Vector2D operator-(const Vector2D& a, const Vector2D& b) {
    return Vector2D(a.x - b.x, a.y - b.y);
}

Vector2D operator*(const Vector2D& v, double scalar) {
    return Vector2D(v.x * scalar, v.y * scalar);
}

Vector2D operator*(double scalar, const Vector2D& v) {
    return Vector2D(v.x * scalar, v.y * scalar);
}

bool operator==(const Vector2D& a, const Vector2D& b) {
    return a.x == b.x && a.y == b.y;
}

double dot(const Vector2D& a, const Vector2D& b) {
    return a.x * b.x + a.y * b.y;
}

// Friend class
class Vector2DFactory {
public:
    Vector2D create_unit_x() {
        return Vector2D(1.0, 0.0);
    }

    Vector2D create_unit_y() {
        return Vector2D(0.0, 1.0);
    }

    Vector2D create_zero() {
        return Vector2D(0.0, 0.0);
    }
};

// Another friend example
class Inner {
private:
    int secret_value;

public:
    Inner(int v) : secret_value(v) {}

    friend class Outer;
};

class Outer {
private:
    Inner inner;

public:
    Outer(int v) : inner(v) {}

    int get_secret() const {
        return inner.secret_value;  // Accessing private member via friend
    }
};
