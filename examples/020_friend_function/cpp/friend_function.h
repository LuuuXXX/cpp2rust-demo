#pragma once

class MyClass {
    int secret_value;
    friend int friend_function_getSum(const MyClass* a, const MyClass* b);
    friend int friend_function_getProduct(const MyClass* a, const MyClass* b);
    friend int friend_function_compare(const MyClass* a, const MyClass* b);
public:
    MyClass(int v);
    ~MyClass();
    int getValue() const;
    void setValue(int v);
};
