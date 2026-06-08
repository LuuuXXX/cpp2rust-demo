# 013_inheritance_single - 单继承

## C++ 特性

本示例展示 C++ 单继承的 FFI 映射。

## C++ 代码

### inheritance_single.h

```cpp
// Animal 基类
struct Animal {
    char name[64];
};

// Dog 派生类 (通过组合模拟继承)
struct Dog {
    struct Animal base;  // 组合：Dog 的基类部分
};
```

### 继承层次

```cpp
class Animal {
    string name;
public:
    string getName();
    void speak();
};

class Dog : public Animal {  // 单继承
public:
    void bark();  // Dog 独有的方法
};
```

## 单继承与 FFI

### 组合替代继承

在 FFI 中，我们使用组合模式模拟继承：

```cpp
// C++ 继承
class Dog : public Animal { };

// FFI 组合
struct Dog {
    struct Animal base;  // Dog 包含 Animal 作为其"基类"部分
};
```

### FFI 映射

| C++ 概念 | FFI 模式 |
|----------|----------|
| 派生类对象 | 包含基类结构体的结构体 |
| 基类方法 | 转发函数接受派生类指针 |
| 继承的方法 | 单独的转发函数 |

### 方法转发

```cpp
// Dog 继承的 Animal 方法需要手动转发
const char* dog_getName(struct Dog* self) {
    return self->base.name;  // 转发到基类部分
}
```

## Rust FFI 代码

```rust
hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <string>

    #include "inheritance_single.h"
}

hicc::import_class! {
    #[cpp(class = "Animal", destroy = "animal_delete")]
    pub class Animal {
        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;

        #[cpp(method = "void speak() const")]
        fn speak(&self);
    }
}

hicc::import_class! {
    #[cpp(class = "Dog", destroy = "dog_delete")]
    pub class Dog {
        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;

        #[cpp(method = "void bark() const")]
        fn bark(&self);

        #[cpp(method = "void speak() const")]
        fn speak(&self);
    }
}

hicc::import_lib! {
    #![link_name = "inheritance_single"]

    class Animal;
    class Dog;

    #[cpp(func = "Animal* animal_new(const char*)")]
    unsafe fn animal_new(name: *const i8) -> Animal;

    #[cpp(func = "Dog* dog_new(const char*)")]
    unsafe fn dog_new(name: *const i8) -> Dog;
}
```
## 关键点

### 继承的挑战

1. **is-a 关系**：派生类是基类的扩展
2. **方法调度**：派生类继承基类的方法
3. **内存布局**：派生类以基类开始

### FFI 组合模式

```cpp
struct Derived {
    struct Base base;  // 必须作为第一个成员
    // 派生类特有的成员
};
```

注意：基类成员必须是第一个，以确保内存布局兼容。

### 命名约定

| C++ 方法 | FFI 函数 |
|----------|----------|
| `dog.getName()` | `dog_getName(dog)` |
| `dog.speak()` | `dog_speak(dog)` |
| `dog.bark()` | `dog_bark(dog)` |

## 运行结果

```
Animal name: Generic Animal
Generic Animal makes a sound

Dog name: Buddy
Buddy barks: Woof! Woof!
Buddy barks: Woof! Woof!

Rust FFI: Single inheritance with hicc pattern
```

## 总结

1. **单继承**：派生类包含基类作为第一个成员
2. **FFI 映射**：通过组合和手动转发实现
3. **方法转发**：继承的方法需要单独的转发函数
