# 015_virtual_basic - 虚函数基础

## C++ 特性

本示例展示 C++ 虚函数的 FFI 映射挑战。

## C++ 代码

### virtual_basic.h

```cpp
class Shape {
public:
    virtual double area() = 0;
    virtual ~Shape() {}
};

class Circle : public Shape {
    double radius;
public:
    Circle(double r) : radius(r) {}
    double area() override { return M_PI * radius * radius; }
};
```

## 虚函数与 Vtable

### C++ 虚函数机制

```cpp
class Shape {
    virtual double area();  // 编译器创建 vtable 指针
};
```

虚函数通过 **vtable（虚表）** 实现：
- 每个类有一个 vtable
- 每个对象有一个 vtable 指针（隐藏在对象布局中）
- 调用虚函数：通过 vtable 指针找到实际实现

```
Shape* s = new Circle(5.0);
s->area();  // 通过 vtable 调用 Circle::area()
```

### Vtable 结构（编译器相关）

```
struct Shape {
    void** vtable;  // 隐藏的 vtable 指针
};

struct Circle {
    void** vtable;  // 继承自 Shape
    double radius;
};
```

## FFI 挑战

### 为什么直接 FFI 虚函数很困难

1. **编译器特定**：vtable 布局是实现细节
2. **内存布局不透明**：指针偏移因编译器而异
3. **多重继承**：vtable 指针可能有多个
4. **RTTI**：运行时类型信息也是编译器特定的

### 可能的解决方案

#### 1. 手动 Vtable 实现

```c
// 定义 vtable 结构
typedef double (*area_fn)(void*);

struct ShapeVtbl {
    area_fn area;
};

struct Shape {
    struct ShapeVtbl* vtbl;
};

// 每个派生类实现自己的 vtable
```

#### 2. 外部调度

```rust
// 不通过 vtable，而是显式分发
enum ShapeType { Circle, Rectangle }
struct ShapeData {
    ShapeType type,
    void* ptr,
}

fn call_area(shape: &ShapeData) -> f64 {
    match shape.type {
        ShapeType::Circle => circle_area(shape.ptr),
        ShapeType::Rectangle => rectangle_area(shape.ptr),
    }
}
```

## Rust FFI 代码

```rust
// 虚函数的 Rust FFI 表示
#[cpp(func = "double shape_area(struct Shape*)")]
unsafe fn shape_area(self_: *mut Shape) -> f64;

#[cpp(func = "double circle_area(struct Circle*)")]
unsafe fn circle_area(self_: *mut Circle) -> f64;
```

注意：这里直接调用 `circle_area` 而不是通过 `shape_area` 的多态调用。

## 关键点

### Vtable 的复杂性

| 问题 | 影响 |
|------|------|
| 编译器特定布局 | 跨编译器二进制不兼容 |
| 多重继承 | 可能有多个 vtable 指针 |
| 虚析构函数 | 需要额外的 vtable 条目 |
| override | 编译器检查，不影响 vtable |

### 实际 FFI 策略

1. **不要暴露原始 C++ 类层次**
2. **使用组合替代继承**
3. **提供独立的工厂函数返回具体类型**
4. **使用枚举标记不同类型**

## 运行结果

```
=== Virtual Function FFI with hicc ===

Circle name: Circle
Circle radius: 5
Circle area: 78.5398

Rust FFI: Virtual functions work through hicc import_class!
```

## 总结

1. **虚函数**：通过 vtable 实现运行时多态
2. **FFI 挑战**：vtable 布局是编译器特定的实现细节
3. **解决方案**：手动实现 vtable 或使用外部调度
4. **最佳实践**：在 FFI 边界避免暴露虚函数层次
