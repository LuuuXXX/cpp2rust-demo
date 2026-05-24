// Example 07: Template Functions
// C++ features: function templates, template specialization, variadic templates

#include <cstring>
#include <cmath>

// Basic template function
template<typename T>
T max_value(T a, T b) {
    return a > b ? a : b;
}

template<typename T>
T min_value(T a, T b) {
    return a < b ? a : b;
}

// Swap template
template<typename T>
void swap_values(T& a, T& b) {
    T temp = a;
    a = b;
    b = temp;
}

// Array sum template
template<typename T, size_t N>
T sum_array(T (&arr)[N]) {
    T total = T();
    for (size_t i = 0; i < N; i++) {
        total += arr[i];
    }
    return total;
}

// Type conversion template
template<typename T, typename U>
U convert(T value) {
    return static_cast<U>(value);
}

// Template with multiple parameters
template<typename T, typename U>
struct Pair {
    T first;
    U second;
};

template<typename T, typename U>
Pair<T, U> make_pair(T a, U b) {
    return Pair<T, U>{a, b};
}

// Variadic template
template<typename T>
T sum(T value) {
    return value;
}

template<typename T, typename... Args>
T sum(T first, Args... args) {
    return first + sum(args...);
}

// Max of variadic
template<typename T>
T max_of(T value) {
    return value;
}

template<typename T, typename... Args>
T max_of(T first, Args... args) {
    T rest = max_of(args...);
    return first > rest ? first : rest;
}

// Template specialization
template<typename T>
T absolute(T value) {
    return value < T() ? -value : value;
}

template<>
const char* absolute(const char* value) {
    return value;
}
