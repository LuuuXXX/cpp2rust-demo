// Example 15: Exception Handling
// C++ features: try/catch, throw, custom exceptions, exception safety

#include <exception>
#include <stdexcept>
#include <string>

// Custom exception class
class MyException : public std::runtime_error {
private:
    int error_code;
public:
    MyException(const std::string& msg, int code)
        : std::runtime_error(msg), error_code(code) {}
    int get_code() const { return error_code; }
};

// Functions that throw
int divide_throw(int a, int b) {
    if (b == 0) {
        throw std::runtime_error("Division by zero");
    }
    return a / b;
}

int get_element(const int* arr, int size, int index) {
    if (index < 0 || index >= size) {
        throw std::out_of_range("Index out of range");
    }
    return arr[index];
}

void throw_custom() {
    throw MyException("Custom error occurred", 42);
}

// Try-catch in functions
int safe_divide(int a, int b, int default_val) {
    try {
        return divide_throw(a, b);
    } catch (const std::exception& e) {
        (void)e;
        return default_val;
    }
}

// Noexcept function
int noexcept_func() noexcept {
    return 42;
}

// Exception in constructor
class ThrowingClass {
private:
    int value;
public:
    ThrowingClass(int v) {
        if (v < 0) {
            throw std::invalid_argument("Value must be non-negative");
        }
        value = v;
    }
    int get() const { return value; }
};
