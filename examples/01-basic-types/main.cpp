// Example 01: Basic Types and Functions
// C++ features: fundamental types, functions, enums, arrays, pointers

#include <cstdint>
#include <cstring>
#include <cmath>

// Global constants
const int MAX_VALUE = 100;
const double PI = 3.14159265358979323846;

// Enum type
enum Color {
    RED = 0,
    GREEN = 1,
    BLUE = 2
};

// Struct for compound types
struct Point3D {
    double x;
    double y;
    double z;
};

// Basic arithmetic functions
int add_int(int a, int b) {
    return a + b;
}

int sub_int(int a, int b) {
    return a - b;
}

int mul_int(int a, int b) {
    return a * b;
}

int div_int(int a, int b) {
    return b != 0 ? a / b : 0;
}

double add_double(double a, double b) {
    return a + b;
}

double sub_double(double a, double b) {
    return a - b;
}

double mul_double(double a, double b) {
    return a * b;
}

double div_double(double a, double b) {
    return b != 0.0 ? a / b : 0.0;
}

// Comparison functions
bool is_greater(int a, int b) {
    return a > b;
}

bool is_equal(int a, int b) {
    return a == b;
}

bool is_less(double a, double b) {
    return a < b;
}

// Min/Max functions
int max_int(int a, int b) {
    return a > b ? a : b;
}

int min_int(int a, int b) {
    return a < b ? a : b;
}

double max_double(double a, double b) {
    return a > b ? a : b;
}

double min_double(double a, double b) {
    return a < b ? a : b;
}

// Type conversion
double int_to_double(int v) {
    return static_cast<double>(v);
}

int double_to_int(double v) {
    return static_cast<int>(v);
}

// Array sum
int sum_array(const int* arr, int len) {
    int sum = 0;
    for (int i = 0; i < len; i++) {
        sum += arr[i];
    }
    return sum;
}

// String length (using C-style string)
int string_length(const char* s) {
    return strlen(s);
}

// Enum to int conversion
int color_to_int(Color c) {
    return static_cast<int>(c);
}

// Struct manipulation
double point_distance(Point3D p) {
    return sqrt(p.x * p.x + p.y * p.y + p.z * p.z);
}

Point3D point_add(Point3D a, Point3D b) {
    Point3D result = {a.x + b.x, a.y + b.y, a.z + b.z};
    return result;
}
