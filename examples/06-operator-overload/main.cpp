// Example 06: Operator Overloading
// C++ features: operator overloading, conversion operators, subscript operator, function call operator

#include <cmath>
#include <cstring>

class Complex {
private:
    double real;
    double imag;

public:
    Complex() : real(0), imag(0) {}
    Complex(double r, double i) : real(r), imag(i) {}

    // Arithmetic operators
    Complex operator+(const Complex& other) const {
        return Complex(real + other.real, imag + other.imag);
    }

    Complex operator-(const Complex& other) const {
        return Complex(real - other.real, imag - other.imag);
    }

    Complex operator*(const Complex& other) const {
        return Complex(
            real * other.real - imag * other.imag,
            real * other.imag + imag * other.real
        );
    }

    Complex operator/(const Complex& other) const {
        double denom = other.real * other.real + other.imag * other.imag;
        return Complex(
            (real * other.real + imag * other.imag) / denom,
            (imag * other.real - real * other.imag) / denom
        );
    }

    // Comparison operators
    bool operator==(const Complex& other) const {
        return real == other.real && imag == other.imag;
    }

    bool operator!=(const Complex& other) const {
        return !(*this == other);
    }

    // Unary operators
    Complex operator-() const {
        return Complex(-real, -imag);
    }

    Complex operator+() const {
        return *this;
    }

    // Access operators
    double& operator[](int idx) {
        if (idx == 0) return real;
        return imag;
    }

    const double& operator[](int idx) const {
        if (idx == 0) return real;
        return imag;
    }

    // Function call operator
    double operator()(double multiplier) const {
        return std::sqrt(real * real + imag * imag) * multiplier;
    }

    // Conversion operator
    explicit operator bool() const {
        return real != 0 || imag != 0;
    }

    double get_real() const { return real; }
    double get_imag() const { return imag; }
};

class String {
private:
    char* data;
    size_t len;

public:
    String() : data(nullptr), len(0) {}

    String(const char* s) {
        len = strlen(s);
        data = new char[len + 1];
        strcpy(data, s);
    }

    String(const String& other) {
        len = other.len;
        data = new char[len + 1];
        strcpy(data, other.data);
    }

    ~String() {
        delete[] data;
    }

    String& operator=(const String& other) {
        if (this != &other) {
            delete[] data;
            len = other.len;
            data = new char[len + 1];
            strcpy(data, other.data);
        }
        return *this;
    }

    String operator+(const String& other) const {
        String result;
        result.len = len + other.len;
        result.data = new char[result.len + 1];
        strcpy(result.data, data);
        strcat(result.data, other.data);
        return result;
    }

    char& operator[](size_t idx) {
        return data[idx];
    }

    const char& operator[](size_t idx) const {
        return data[idx];
    }

    const char* c_str() const { return data; }
    size_t length() const { return len; }
};
