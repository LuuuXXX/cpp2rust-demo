// Example 13: Lambdas and Functional Programming
// C++ features: lambda expressions, std::function, closures, capture groups

#include <functional>
#include <vector>
#include <algorithm>
#include <iostream>

// Function that takes a lambda
int apply(int x, int y, std::function<int(int, int)> op) {
    return op(x, y);
}

// Function that returns a lambda
std::function<int(int)> make_multiplier(int factor) {
    return [factor](int x) {
        return x * factor;
    };
}

// Function with captured lambda
auto make_counter(int start) {
    int count = start;
    return [count]() mutable {
        return count++;
    };
}

// Generic algorithm with lambda
int sum_if(const std::vector<int>& v, std::function<bool(int)> pred) {
    int sum = 0;
    for (int x : v) {
        if (pred(x)) {
            sum += x;
        }
    }
    return sum;
}

// Lambda with capture by reference
void modify_by_ref(int& x, int& y, std::function<void()> operation) {
    operation();
}

// Variadic lambda
auto make_adder(int x) {
    return [x](auto... ys) {
        return (x + ... + ys);
    };
}
