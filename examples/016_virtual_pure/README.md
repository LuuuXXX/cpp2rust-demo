# 016_virtual_pure - 纯虚函数/抽象类

## C++ 特性

本示例展示 C++ 纯虚函数和抽象类的 FFI 映射。

## C++ 代码

### virtual_pure.h

```cpp
// 抽象基类 - 不能直接实例化
class AbstractShape {
public:
    // 纯虚函数
    virtual double area() = 0;
    virtual const char* getName() = 0;
    virtual ~AbstractShape() {}
};

// 具体实现
class Circle : public AbstractShape {
    double radius;
public:
    Circle(double r) : radius(r) {}
    double area() override { return M_PI * radius * radius; }
    const char* getName() override { return "Circle"; }
};

class Rectangle : public AbstractShape {
    double width, height;
public:
    Rectangle(double w, double h) : width(w), height(h) {}
    double area() override { return width * height; }
    const char* getName() override { return "Rectangle"; }
};
```

## 纯虚函数与抽象类

### C++ 纯虚函数

```cpp
virtual double area() = 0;  // = 0 表示纯虚函数
```

纯虚函数的特性：
1. **没有实现**：只有声明
2. **使类抽象**：包含纯虚函数的类不能实例化
3. **强制覆盖**：派生类必须实现所有纯虚函数

### 抽象类的限制

```cpp
AbstractShape s;     // 错误！抽象类不能实例化
AbstractShape* p;    // 正确，可以有抽象类指针
p = new Circle(5);   // 正确，指向具体实现
```

## FFI 映射

### 函数指针模拟纯虚函数

```c
// 纯虚函数的 C 实现
typedef double (*area_fn)(struct AbstractShape*);

struct AbstractShape {
    area_fn area;       // 函数指针 = 纯虚函数
    const char* name;
    double dim1;
    double dim2;
};
```

### 工厂模式

```c
// 抽象类不能实例化，通过工厂函数返回具体实现
struct AbstractShape* create_circle(double radius);
struct AbstractShape* create_rectangle(double width, double height);
```

### 多态调用

```c
// 通过函数指针调用 - 实现多态
double calculate(struct AbstractShape* shape) {
    return shape->area(shape);  // 调用实际实现
}
```

## Rust FFI 代码

```rust
hicc::cpp! {
    #include <iostream>
    #include <cmath>
    #include <cstring>

    #include "virtual_pure.h"
}

hicc::import_class! {
    #[cpp(class = "AbstractShape", destroy = "abstract_shape_delete")]
    pub class AbstractShape {
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "virtual_pure"]

    class AbstractShape;

    #[cpp(func = "AbstractShape* abstract_shape_create_circle(double)")]
    fn abstract_shape_create_circle(radius: f64) -> *mut AbstractShape;

    #[cpp(func = "AbstractShape* abstract_shape_create_rectangle(double, double)")]
    fn abstract_shape_create_rectangle(width: f64, height: f64) -> *mut AbstractShape;
}
```
## 关键点

### 抽象类的 FFI 策略

1. **函数指针替代 vtable**：手动实现虚表
2. **工厂函数**：返回具体实现（而非抽象类实例）
3. **统一接口**：所有具体实现通过相同函数指针签名

### 多态的实现

```c
// 相同接口，不同实现
struct AbstractShape* shapes[2];
shapes[0] = create_circle(5.0);
shapes[1] = create_rectangle(4.0, 6.0);

for (int i = 0; i < 2; i++) {
    printf("Area: %f\n", shapes[i]->area(shapes[i]));
}
```

### 内存布局

```
Circle:
  area_fn area ---------> circle_area_impl
  const char* name ----> "Circle"
  double dim1 ----------> 5.0 (radius)

Rectangle:
  area_fn area ---------> rectangle_area_impl
  const char* name ----> "Rectangle"
  double dim1 ----------> 4.0 (width)
  double dim2 ----------> 6.0 (height)
```

## 运行结果

```
=== Pure Virtual Function FFI with hicc ===

Pure virtual functions (= 0) make a class abstract
Cannot be instantiated directly in C++


--- Using circle through abstract interface ---
Shape: Circle
Area: 78.5398

--- Using rectangle through abstract interface ---
Shape: Rectangle
Area: 24.0000

--- Polymorphic behavior demonstrated ---
Deleting Circle
Deleting Circle
Deleting Rectangle
Deleting Rectangle

Rust FFI: Pure virtual functions work through hicc!
```

## 总结

1. **纯虚函数**：`= 0` 语法，不能有实现
2. **抽象类**：包含纯虚函数的类，不能实例化
3. **FFI 映射**：函数指针模拟 vtable
4. **多态**：通过工厂返回具体类型，通过统一接口调用
