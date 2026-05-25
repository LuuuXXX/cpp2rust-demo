#pragma once

#ifdef __cplusplus
extern "C" {
#endif

struct MyClass;

struct MyClass* myclass_new(int secret_value);
void myclass_delete(struct MyClass* self);

int myclass_getValue(struct MyClass* self);
void myclass_setValue(struct MyClass* self, int value);

int friend_function_getSum(const struct MyClass* a, const struct MyClass* b);
int friend_function_getProduct(const struct MyClass* a, const struct MyClass* b);
int friend_function_compare(const struct MyClass* a, const struct MyClass* b);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
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

#endif
