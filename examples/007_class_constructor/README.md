# 007_class_constructor - 构造函数重载

## C++ 特性

本示例展示 C++ 类的构造函数重载，通过不同的工厂函数实现。

## C++ 代码

### class_constructor.h

```cpp
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

struct Point;

// 构造函数变体
struct Point* point_new_xy(int x, int y);
struct Point* point_newPolar(double r, double theta);
void point_delete(struct Point* self);

int point_getX(struct Point* self);
int point_getY(struct Point* self);
double point_getMagnitude(struct Point* self);
double point_getAngle(struct Point* self);

#ifdef __cplusplus
}
#endif
```

### class_constructor.cpp

```cpp
#include "class_constructor.h"
#include <cmath>

struct Point {
    int x;
    int y;
};

struct Point* point_new_xy(int x, int y) {
    std::cout << "Point created: xy(" << x << ", " << y << ")" << std::endl;
    return new Point{x, y};
}

struct Point* point_newPolar(double r, double theta) {
    int x = static_cast<int>(r * cos(theta));
    int y = static_cast<int>(r * sin(theta));
    std::cout << "Point created: polar(" << r << ", " << theta << ")" << std::endl;
    return new Point{x, y};
}
```

## 构造函数重载与 FFI

### C++ 构造函数重载

C++ 允许同名构造函数，根据参数类型和个数选择：

```cpp
class Point {
public:
    Point(int x, int y);           // 构造函数 #1
    Point(double r, double theta); // 构造函数 #2
};
```

### FFI 映射策略

由于 C 没有函数重载，需要通过不同的函数名实现：

| C++ 构造函数 | FFI 函数 |
|--------------|----------|
| `Point(int x, int y)` | `point_new_xy(int x, int y)` |
| `Point(double r, double theta)` | `point_newPolar(double r, double theta)` |

## Rust FFI 代码

```rust
hicc::cpp! {
    #include <iostream>
    #include <cmath>

    #include "class_constructor.h"
}

hicc::import_class! {
    #[cpp(class = "Point", destroy = "point_delete")]
    pub class Point {
        #[cpp(method = "int getX() const")]
        fn get_x(&self) -> i32;

        #[cpp(method = "int getY() const")]
        fn get_y(&self) -> i32;

        #[cpp(method = "double getMagnitude() const")]
        fn get_magnitude(&self) -> f64;

        #[cpp(method = "double getAngle() const")]
        fn get_angle(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "class_constructor"]

    class Point;

    #[cpp(func = "Point* point_new_xy(int, int)")]
    fn point_new_xy(x: i32, y: i32) -> Point;

    #[cpp(func = "Point* point_newPolar(double, double)")]
    fn point_new_polar(r: f64, theta: f64) -> Point;
}
```
## 关键点

### 构造函数重载的 FFI 策略

1. **工厂函数命名**：使用有意义的名称（如 `point_new_xy`）区分不同构造方式
2. **参数类型明确**：每个重载的参数类型必须不同
3. **Rust 端无重载**：Rust 端通过不同的函数名调用

### 构造函数初始化顺序

C++ 构造函数体内，初始化列表先于构造函数体执行。在 FFI 中：

```rust
// C++: Point(x, y) { }  等价于
fn point_new_xy(x: i32, y: i32) -> *mut Point {
    // 1. 分配内存
    // 2. 初始化成员
    // 3. 返回指针
}
```

## 构建方法

### C++ 编译

```bash
cd cpp
g++ -c class_constructor.cpp -o class_constructor.o
g++ -shared -fPIC class_constructor.cpp -o libclass_constructor.so
```

### Rust 编译

```bash
cd rust_hicc
cargo build
cargo run
```

## 运行结果

```
Point 1: (3, 4)
  Magnitude: 5
  Angle: 0.9272952180016122

Point created: polar(5, 0) -> xy(5, 0)
Point 2: (5, 0)
  Magnitude: 5
  Angle: 0

Rust FFI: Multiple constructors work!
```

## 总结

1. **构造函数重载不能直接映射**：C 没有重载
2. **使用工厂函数**：每个构造函数对应一个工厂函数
3. **命名体现语义**：`point_new_xy` vs `point_newPolar`
4. **Rust 端无重载**：通过不同函数名区分
