#include "exception_basic.h"
#include <iostream>

int main() {
    using namespace exception_basic_ns;

    Calculator calc;
    std::cout << "10 / 2 = " << calc.divide(10, 2)
              << " error=" << calc.last_error() << "\n";
    std::cout << "1 / 0 = " << calc.divide(1, 0)
              << " error=" << calc.last_error()
              << " has_error=" << calc.has_error() << "\n";
    calc.clear_error();
    std::cout << "after clear has_error=" << calc.has_error() << "\n";

    std::cout << "parse_int(123) = " << calc.parse_int("123")
              << " error=" << calc.last_error() << "\n";
    std::cout << "parse_int(abc) = " << calc.parse_int("abc")
              << " error=" << calc.last_error() << "\n";
    std::cout << "parse_int(huge) = " << calc.parse_int("99999999999999999999")
              << " error=" << calc.last_error() << "\n";
    return 0;
}
