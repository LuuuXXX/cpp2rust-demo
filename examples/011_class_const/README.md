# 011_class_const - const 成员函数

## C++ 特性

本示例展示 C++ const 成员函数的 FFI 映射。

## C++ 代码

### class_const.h

```cpp
// const 成员函数
int calculator_getValue(const struct Calculator* self);
int calculator_getHistoryCount(const struct Calculator* self);

// 非 const 成员函数
void calculator_add(struct Calculator* self, int value);
```

### const 成员函数

```cpp
int calculator_getValue(const struct Calculator* self) const {
    return self->value;  // 不能修改成员
}
```

## const 成员函数与 FFI

### C++ const 成员函数

```cpp
class Calculator {
    int getValue() const;  // const 版本
    void add(int value);   // 非 const 版本
};
```

const 成员函数承诺不修改对象状态。

### FFI 映射

| C++ 函数 | FFI 声明 |
|----------|----------|
| `int get() const` | `int get(const Class*)` |
| `void add(int)` | `void add(Class*, int)` |

关键区别：`this` 参数是否 const。

## Rust FFI 代码

```rust
hicc::cpp! {
    #include <iostream>
    #include <vector>

    #include "class_const.h"
}

hicc::import_class! {
    #[cpp(class = "Calculator", destroy = "calculator_delete")]
    pub class Calculator {
        #[cpp(method = "int getValue() const")]
        fn get_value(&self) -> i32;

        #[cpp(method = "int getHistoryCount() const")]
        fn get_history_count(&self) -> i32;

        #[cpp(method = "void add(int v)")]
        fn add(&mut self, v: i32);

        #[cpp(method = "void subtract(int v)")]
        fn subtract(&mut self, v: i32);

        #[cpp(method = "void clear()")]
        fn clear(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "class_const"]

    class Calculator;

    #[cpp(func = "Calculator* calculator_new()")]
    fn calculator_new() -> Calculator;
}
```
## 关键点

### const 正确性

使用 const 成员函数可以获得：
1. **编译时检查**：编译器防止意外修改
2. **API 明确性**：调用者知道函数不会修改状态
3. **重载基础**：const 和非 const 版本可以重载

### Rust 中的等价物

| C++ | Rust |
|-----|------|
| `const Class*` | `*const Class` |
| `Class*` | `*mut Class` |
| const 成员函数 | 接受 `*const T` 的函数 |

## 运行结果

```
Initial value: 0
History count: 0
After add(10): 10
After add(5): 15
After subtract(3): 12
History count: 3
After clear: 0
History count: 0

Rust FFI: const member functions work!
```

## 总结

1. **const 成员函数**：承诺不修改对象
2. **FFI 映射**：`this` 参数变为 `const Class*`
3. **Rust 类型安全**：const 版本接受 `*const T`
