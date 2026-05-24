// Example 20: Template Specialization
// C++ features: full specialization, partial specialization, SFINAE concepts

#include <cstring>
#include <type_traits>

// Primary template
template<typename T>
class Wrapper {
private:
    T value;

public:
    Wrapper(T v) : value(v) {}
    T get() const { return value; }
};

// Full specialization for int
template<>
class Wrapper<int> {
private:
    int value;

public:
    Wrapper(int v) : value(v) {}
    int get() const { return value; }
    int get_squared() const { return value * value; }  // int-specific method
};

// Partial specialization for pointers
template<typename T>
class Wrapper<T*> {
private:
    T* value;

public:
    Wrapper(T* v) : value(v) {}
    T* get() const { return value; }
    bool is_null() const { return value == nullptr; }
};

// Partial specialization for const types
template<typename T>
class Wrapper<const T> {
private:
    T value;

public:
    Wrapper(T v) : value(v) {}
    T get() const { return value; }
};

// Template function specialization
template<typename T>
T identity(T v) {
    return v;
}

template<>
const char* identity(const char* v) {
    return v;
}

// Variadic template with specialization
template<typename... Args>
class Tuple {
    // Primary - empty
};

template<typename T, typename... Args>
class Tuple<T, Args...> : public Tuple<Args...> {
private:
    T first;

public:
    Tuple(T f, Args... rest) : Tuple<Args...>(rest...), first(f) {}
    T get_first() const { return first; }
};

// SFINAE example
template<typename T>
class IsPointer {
public:
    static constexpr bool value = false;
};

template<typename T>
class IsPointer<T*> {
public:
    static constexpr bool value = true;
};

template<typename T>
typename std::enable_if<!IsPointer<T>::value, T>::type
smart_get(T v) {
    return v;
}

template<typename T>
typename std::enable_if<IsPointer<T>::value, T>::type
smart_get(T v) {
    return v;
}
