# 020_friend_function - 友元函数

## C++ 特性

本示例展示 C++ 友元函数的 FFI 映射。

## C++ 代码

### friend_function.h

```cpp
class MyClass {
    int secret_value;  // private 成员
public:
    MyClass(int v);
    int getValue();
    void setValue(int v);

    // 友元函数声明
    friend int getSum(const MyClass& a, const MyClass& b);
    friend int getProduct(const MyClass& a, const MyClass& b);
    friend int compare(const MyClass& a, const MyClass& b);
};

// 友元函数定义
inline int getSum(const MyClass& a, const MyClass& b) {
    return a.secret_value + b.secret_value;  // 可以访问 private
}
```

## 友元函数

### C++ 友元机制

```cpp
class MyClass {
    int value;  // private
public:
    friend void friendFunc(MyClass* obj);  // 声明友元
};

void friendFunc(MyClass* obj) {
    obj->value = 10;  // 可以访问 private 成员
}
```

友元的特性：
1. **访问权限**：友元可以访问类的所有 private 成员
2. **不是成员**：友元函数不是类的成员
3. **单向关系**：A 是 B 的友元，不意味着 B 是 A 的友元
4. **可传递**：A 是 B 的友元，B 是 C 的友元，不意味着 A 是 C 的友元

### 友元的类型

| 类型 | 说明 |
|------|------|
| 友元函数 | 普通函数获得访问权限 |
| 友元类 | 另一个类的所有成员获得访问权限 |
| 友元成员函数 | 另一个类的特定成员函数获得访问权限 |

## FFI 映射

### 友元函数 vs 普通函数

在 FFI 中，友元函数和普通函数没有区别：

```c
// C FFI 中，友元函数就是普通函数
// 结构体定义对 C 是透明的（没有访问控制）

int friend_function_getSum(struct MyClass* a, struct MyClass* b) {
    return a->secret_value + b->secret_value;  // 直接访问
}
```

### C++ 访问控制 vs C FFI

| C++ | C FFI |
|-----|-------|
| private 成员 | 结构体成员（可直接访问） |
| 友元函数 | 普通函数 |
| protected | 不存在（用 C 结构体模拟） |

## Rust FFI 代码

```rust
hicc::cpp! {
    #include <iostream>

    #include "friend_function.h"
}

hicc::import_class! {
    #[cpp(class = "MyClass", destroy = "myclass_delete")]
    pub class MyClass {
        #[cpp(method = "int getValue() const")]
        fn get_value(&self) -> i32;

        #[cpp(method = "void setValue(int v)")]
        fn set_value(&mut self, v: i32);
    }
}

hicc::import_lib! {
    #![link_name = "friend_function"]

    class MyClass;

    #[cpp(func = "MyClass* myclass_new(int)")]
    fn myclass_new(secret_value: i32) -> MyClass;

    #[cpp(func = "int friend_function_getSum(const MyClass* a, const MyClass* b)")]
    fn friend_function_get_sum(a: *const MyClass, b: *const MyClass) -> i32;

    #[cpp(func = "int friend_function_getProduct(const MyClass* a, const MyClass* b)")]
    fn friend_function_get_product(a: *const MyClass, b: *const MyClass) -> i32;

    #[cpp(func = "int friend_function_compare(const MyClass* a, const MyClass* b)")]
    fn friend_function_compare(a: *const MyClass, b: *const MyClass) -> i32;
}
```
## 关键点

### 为什么使用友元

1. **封装优于友元**：优先使用 public 接口
2. **运算符重载**：需要访问 private 成员（如 operator<<）
3. **类协作**：两个类需要互相访问对方的 private

### 示例：运算符重载作为友元

```cpp
class Number {
    int value;
public:
    Number(int v) : value(v) {}
    friend Number operator+(const Number& a, const Number& b);
};

Number operator+(const Number& a, const Number& b) {
    return Number(a.value + b.value);  // 访问 private
}
```

### FFI 中的友元

```c
// 在 C FFI 中不需要"友元"概念
// 因为 C 没有访问控制
struct Number {
    int value;  // 总是可访问的
};

struct Number* number_add(struct Number* a, struct Number* b) {
    return number_new(a->value + b->value);
}
```

## 运行结果

```
=== Friend Function FFI ===

Friend functions in C++ can access private members of a class

Created MyClass objects:
  a.value = 10
  b.value = 20

Friend function operations:
Friend function getSum: 10 + 20 = 30
  Sum: 30
Friend function getProduct: 10 * 20 = 200
  Product: 200
Friend function compare: a < b
  Compare: -1

Rust FFI: Friend functions are just regular functions
In C FFI, we can access struct members directly
The 'friend' relationship is a C++ access control concept
```

## 总结

1. **友元函数**：获得访问 private 成员的普通函数
2. **FFI 映射**：在 C FFI 中就是普通函数
3. **访问控制**：C 没有访问控制，结构体成员都可访问
4. **Rust 替代**：Rust 没有友元，但可以通过 `pub(in crate)` 控制可见性
