# 019_operator_overload - 运算符重载

## C++ 特性

本示例展示 C++ 运算符重载的 FFI 映射。

## C++ 代码

### operator_overload.h

```cpp
class Number {
    int value;
public:
    Number(int v) : value(v) {}

    // 运算符重载
    Number operator+(const Number& other) const;
    Number operator-(const Number& other) const;
    Number operator*(const Number& other) const;
    Number operator/(const Number& other) const;

    // 比较运算符
    int operator<=>(const Number& other) const;  // C++20 spaceship

    // 一元运算符
    Number operator-() const;
    Number& operator++();    // 前置++
    Number& operator--();    // 前置--

    // 复合赋值
    Number& operator+=(const Number& other);
    Number& operator-=(const Number& other);
};
```

## 运算符重载与 FFI

### C++ 运算符语法

```cpp
Number a(10), b(3);
Number c = a + b;        // 运算符
Number d = a.operator+(b);  // 成员函数语法
```

### FFI 映射：运算符到命名方法

| C++ 运算符 | C++ 函数语法 | FFI 函数 |
|------------|--------------|----------|
| `a + b` | `a.operator+(b)` | `number_add(a, b)` |
| `a - b` | `a.operator-(b)` | `number_sub(a, b)` |
| `a * b` | `a.operator*(b)` | `number_mul(a, b)` |
| `a / b` | `a.operator/(b)` | `number_div(a, b)` |
| `-a` | `a.operator-()` | `number_negate(a)` |
| `++a` | `a.operator++()` | `number_increment(a)` |
| `a += b` | `a.operator+=(b)` | `number_add_assign(a, b)` |

### 运算符重载规则

1. **不能改变优先级**：`*` 仍然比 `+` 高
2. **不能改变结合性**：`a + b + c` 仍然是 `(a + b) + c`
3. **不能发明新运算符**：只能重载已有的
4. **不能重载内建类型**：`int + int` 不能改变

### C++ 运算符表

| 类别 | 运算符 |
|------|--------|
| 算术 | `+`, `-`, `*`, `/`, `%` |
| 位 | `&`, `\|`, `^`, `~`, `<<`, `>>` |
| 比较 | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| 逻辑 | `&&`, `\|\|`, `!` |
| 赋值 | `=`, `+=`, `-=`, `*=`, `/=` |
| 递增递减 | `++`, `--` |
| 其他 | `[]`, `()`, `->`, `->*`, `,` |

## Rust FFI 代码

```rust
hicc::cpp! {
    #include <iostream>

    #include "operator_overload.h"

    int number_get_value(const Number* self) {
        return self->getValue();
    }

    Number* number_add(const Number* a, const Number* b) {
        return new Number(*a + *b);
    }

    Number* number_sub(const Number* a, const Number* b) {
        return new Number(*a - *b);
    }

    Number* number_mul(const Number* a, const Number* b) {
        return new Number(*a * *b);
    }

    Number* number_div(const Number* a, const Number* b) {
        return new Number(*a / *b);
    }

    Number* number_negate(const Number* a) {
        return new Number(-*a);
    }

    int number_compare(const Number* a, const Number* b) {
        return a->compare(*b);
    }
}

hicc::import_class! {
    #[cpp(class = "Number", destroy = "number_delete")]
    pub class Number {
        #[cpp(method = "int getValue() const")]
        fn get_value(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "operator_overload"]

    class Number;

    #[cpp(func = "Number* number_new(int)")]
    fn number_new(value: i32) -> Number;

    #[cpp(func = "int number_get_value(const Number*)")]
    fn number_getValue(self_: *const Number) -> i32;

    #[cpp(func = "Number* number_add(const Number*, const Number*)")]
    fn number_add(a: *const Number, b: *const Number) -> *mut Number;

    #[cpp(func = "Number* number_sub(const Number*, const Number*)")]
    fn number_sub(a: *const Number, b: *const Number) -> *mut Number;

    #[cpp(func = "Number* number_mul(const Number*, const Number*)")]
    fn number_mul(a: *const Number, b: *const Number) -> *mut Number;

    #[cpp(func = "Number* number_div(const Number*, const Number*)")]
    fn number_div(a: *const Number, b: *const Number) -> *mut Number;

    #[cpp(func = "Number* number_negate(const Number*)")]
    fn number_negate(a: *const Number) -> *mut Number;

    #[cpp(func = "int number_compare(const Number*, const Number*)")]
    fn number_compare(a: *const Number, b: *const Number) -> i32;
}
```
## 关键点

### 运算符重载的 FFI 策略

1. **命名方法**：每个运算符对应一个命名方法
2. **函数签名**：运算符变成普通函数
3. **返回新对象**：大多数运算符返回新对象

### 示例

```cpp
// C++
Number c = a + b * d;

// FFI 等价
Number* temp = number_mul(b, d);
Number* c = number_add(a, temp);
number_delete(temp);
```

### Rust 中的运算符

Rust 也有运算符重载，通过 `std::ops` trait：

```rust
use std::ops::Add;

impl Add for Number {
    type Output = Number;
    fn add(self, other: Number) -> Number {
        Number(self.value + other.value)
    }
}
```

## 运行结果

```
=== Operator Overload FFI ===

C++ operator overloading becomes named method calls in FFI

Created numbers: a = 10, b = 3

Result of a + b = 13
Result of a - b = 7
Result of a * b = 30
Result of a / b = 3

Unary operators:
Negation of a = -10
a compared to b = 7

Rust FFI: Operators become named methods
a + b -> number_add(a, b)
a - b -> number_sub(a, b)
a * b -> number_mul(a, b)
```

## 总结

1. **运算符重载**：让类支持运算符操作
2. **FFI 映射**：运算符变成命名方法调用
3. **命名约定**：`operator+` -> `add`, `operator*` -> `mul`
4. **Rust 替代**：使用 trait 实现运算符重载
