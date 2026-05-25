# 示例 05：虚函数与多态

## 特性概述

本示例展示 C++ 的**虚函数与运行时多态**，包括虚函数声明、纯虚函数、`override` 关键字、动态分派以及工厂模式。多态是 C++ 面向对象设计的核心机制，hicc 通过 `#[interface]` 宏和 `@make_proxy` 内置函数支持完整的双向多态。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| 虚函数 `virtual` | 支持运行时动态分派 |
| 纯虚函数 `= 0` | 定义抽象接口，强制子类实现 |
| `override` 关键字 | 显式标记覆盖基类虚函数 |
| 虚析构函数 | 保证通过基类指针正确销毁对象 |
| 工厂模式 | 纯虚 `create()` 工厂类 |
| 动态分派 | 通过基类指针/引用调用派生类实现 |

### 代码结构

```cpp
// 抽象基类
class Shape {
    virtual double area() const = 0;      // 纯虚
    virtual double perimeter() const = 0; // 纯虚
    virtual void describe() const;        // 有默认实现的虚函数
};

// 具体实现类
class Rectangle : public Shape {
    double area() const override;
    void describe() const override;
};

// 工厂基类
class ShapeFactory {
    virtual Shape* create() const = 0;    // 纯虚工厂方法
};

// 多态函数
double total_area(const Shape** shapes, int n);
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 节点与属性：

| AST 节点 / 属性 | 含义 |
|-----------------|------|
| `CXXMethodDecl.isVirtual: true` | 虚函数 |
| `CXXMethodDecl.isPure: true` | 纯虚函数（必须重写） |
| `CXXDestructorDecl.isVirtual: true` | 虚析构函数 |
| `CXXRecordDecl` 含 `isPure` 方法 | 抽象类 |
| `bases[]` | 继承关系 |

AST 片段示例（纯虚函数）：

```json
{
  "kind": "CXXMethodDecl",
  "name": "area",
  "type": { "qualType": "double () const" },
  "isVirtual": true,
  "isPure": true
}
```

## hicc 处理方式

### 抽象类 → Rust Trait

含纯虚函数的类通过 `#[interface]` 映射为 Rust trait：

```rust
hicc::import_class! {
    #[interface]
    class Shape {
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "double perimeter() const")]
        fn perimeter(&self) -> f64;

        #[cpp(method = "void describe() const")]
        fn describe(&self);
    }
}
```

生成的 Rust 代码：
```rust
pub trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
    fn describe(&self);
}
```

### 具体实现类

```rust
hicc::import_class! {
    #[cpp(class = "Rectangle", ctor = "Rectangle(double, double)")]
    class Rectangle: Shape {
        // 继承 Shape 的所有方法，可额外添加 Rectangle 特有方法
    }
}
```

### Rust 实现 C++ 接口（双向多态）

通过 `@make_proxy` 内置函数，可以让 Rust 类型实现 C++ 接口：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    class Shape;

    #[cpp(func = "Shape @make_proxy<Shape>()")]
    #[interface(name = "Shape")]
    fn new_rust_shape(intf: hicc::Interface<Shape>) -> Shape;
}

struct MyRustShape { width: f64, height: f64 }

impl Shape for MyRustShape {
    fn area(&self) -> f64 { self.width * self.height }
    fn perimeter(&self) -> f64 { 2.0 * (self.width + self.height) }
    fn describe(&self) { println!("RustShape {}x{}", self.width, self.height); }
}

// 使用：将 Rust 对象作为 C++ Shape 传递
let shape = new_rust_shape(MyRustShape { width: 4.0, height: 3.0 });
```

### 工厂方法模式

```rust
hicc::import_class! {
    #[interface]
    class ShapeFactory {
        #[cpp(method = "Shape* create() const")]
        fn create(&self) -> Shape;
    }

    #[cpp(class = "RectangleFactory", ctor = "RectangleFactory()")]
    class RectangleFactory: ShapeFactory {}
}
```

### 多态函数调用

```rust
hicc::import_lib! {
    #![link_name = "example"]

    class Shape;

    #[cpp(func = "double total_area(const Shape**, int)")]
    fn total_area(shapes: *const *const Shape, n: i32) -> f64;
}
```

## 注意事项

1. **`#[interface]` 约束**：只能包含 C++ 虚函数（含纯虚），不能包含普通成员方法
2. **虚析构函数**：抽象类必须有虚析构函数，否则通过基类指针删除会产生未定义行为
3. **`@make_proxy` 要求**：使用 `@make_proxy` 的类必须在 `#[cpp(class = ..., ctor = ...)]` 中指定构造函数
4. **多态与所有权**：通过 `Interface<T>` 传递给 C++ 的 Rust 对象，其生命周期必须比 C++ 对象长
5. **cpp2rust-demo 生成策略**：当 AST 检测到类含 `isPure: true` 的方法时自动生成 `#[interface]` trait
