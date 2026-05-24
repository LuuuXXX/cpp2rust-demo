// Example 14: Type Casting
// C++ features: static_cast, dynamic_cast, const_cast, reinterpret_cast

#include <cstdint>

class Base {
public:
    virtual ~Base() {}
    virtual int get_value() const { return 0; }
};

class Derived : public Base {
private:
    int value;
public:
    Derived(int v) : value(v) {}
    int get_value() const override { return value; }
    int get_derived_value() const { return value * 2; }
};

class OtherClass {
public:
    int other_value;
};

// Static cast examples
int static_cast_example(int x) {
    return static_cast<int>(x * 1.5);
}

double static_cast_double(int x) {
    return static_cast<double>(x);
}

// Dynamic cast example
Base* create_derived(int v) {
    return new Derived(v);
}

Derived* to_derived(Base* p) {
    return dynamic_cast<Derived*>(p);
}

// Const cast example
int* const_cast_example(const int* p) {
    return const_cast<int*>(p);
}

// Reinterpret cast examples
intptr_t reinterpret_intptr(int* p) {
    return reinterpret_cast<intptr_t>(p);
}

int* reinterpret_ptr(intptr_t v) {
    return reinterpret_cast<int*>(v);
}

// Void pointer cast
void* to_void_ptr(int* p) {
    return reinterpret_cast<void*>(p);
}

int* from_void_ptr(void* p) {
    return reinterpret_cast<int*>(p);
}
