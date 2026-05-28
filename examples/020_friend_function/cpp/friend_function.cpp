#include "friend_function.h"
#include <iostream>

struct MyClass* myclass_new(int secret_value) {
    return new MyClass(secret_value);
}

void myclass_delete(struct MyClass* self) {
    delete self;
}

int myclass_getValue(struct MyClass* self) {
    return self->getValue();
}

void myclass_setValue(struct MyClass* self, int value) {
    self->setValue(value);
}

int friend_function_getSum(const MyClass* a, const MyClass* b) {
    int sum = a->getValue() + b->getValue();
    std::cout << "Friend function getSum: " << a->getValue()
              << " + " << b->getValue() << " = " << sum << std::endl;
    return sum;
}

int friend_function_getProduct(const MyClass* a, const MyClass* b) {
    int product = a->getValue() * b->getValue();
    std::cout << "Friend function getProduct: " << a->getValue()
              << " * " << b->getValue() << " = " << product << std::endl;
    return product;
}

int friend_function_compare(const MyClass* a, const MyClass* b) {
    if (a->getValue() < b->getValue()) {
        std::cout << "Friend function compare: a < b" << std::endl;
        return -1;
    } else if (a->getValue() > b->getValue()) {
        std::cout << "Friend function compare: a > b" << std::endl;
        return 1;
    } else {
        std::cout << "Friend function compare: a == b" << std::endl;
        return 0;
    }
}

// MyClass implementation
MyClass::MyClass(int v) : secret_value(v) {}
MyClass::~MyClass() {}
int MyClass::getValue() const { return secret_value; }
void MyClass::setValue(int v) { secret_value = v; }
