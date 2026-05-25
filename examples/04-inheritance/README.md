# 示例 04：继承

## 特性概述

本示例展示 C++ 的**继承体系**，包括单继承、多继承、虚继承（解决菱形继承问题）以及访问说明符（`public`/`protected`/`private` 继承）。继承是 C++ 代码复用的重要机制，hicc 通过基类列表和 `#[interface]` 宏来处理。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| 单继承 | `class Derived : public Base` |
| 多继承 | `class ColoredRectangle : public Rectangle, public Color` |
| 虚继承 | `class Mammal : virtual public Animal`（解决菱形继承） |
| 纯虚函数 | `virtual double area() const = 0` |
| 覆盖 `override` | `double area() const override` |
| 访问控制 | 继承的 `public`/`protected` 访问说明符 |

### 代码结构

```cpp
// 抽象基类（含纯虚函数）
class Shape {
    virtual double area() const = 0;
    virtual double perimeter() const = 0;
};

// 单继承
class Rectangle : public Shape { ... };
class Circle : public Shape { ... };

// 多继承
class Color { ... };
class ColoredRectangle : public Rectangle, public Color { ... };

// 虚继承（菱形问题）
class Animal { virtual void speak() const = 0; };
class Mammal : virtual public Animal { ... };
class Bird : virtual public Animal { ... };
class Bat : public Mammal, public Bird { ... };
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 节点：

| AST 节点 / 属性 | 含义 |
|-----------------|------|
| `CXXRecordDecl.bases` | 基类列表（顶层数组，不在 `inner` 中） |
| `bases[].type.qualType` | 基类的完整限定类型名 |
| `bases[].isVirtual` | 是否为虚继承 |
| `bases[].access` | 继承访问方式（`public`/`protected`/`private`） |
| `CXXMethodDecl.isPure` | 纯虚函数标志 |
| `CXXMethodDecl.isVirtual` | 虚函数标志 |

AST 片段示例（基类列表）：

```json
{
  "kind": "CXXRecordDecl",
  "name": "ColoredRectangle",
  "bases": [
    { "access": "public", "type": { "qualType": "Rectangle" } },
    { "access": "public", "type": { "qualType": "Color" } }
  ]
}
```

> **注意**：Clang AST JSON 将 `bases` 放在节点顶层，而非 `inner` 数组中。

## hicc 处理方式

### 抽象基类 → `#[interface]`

含纯虚函数的基类通过 `#[interface]` 标记，生成 Rust trait：

```rust
hicc::import_class! {
    #[interface]
    class Shape {
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "double perimeter() const")]
        fn perimeter(&self) -> f64;
    }
}
```

### 单继承映射

继承关系在 `import_class!` 中通过冒号语法表示：

```rust
hicc::import_class! {
    #[interface]
    class Shape { ... }

    #[cpp(class = "Rectangle", ctor = "Rectangle(double, double)")]
    class Rectangle: Shape {
        #[cpp(method = "double get_width() const")]
        fn get_width(&self) -> f64;

        #[cpp(method = "double get_height() const")]
        fn get_height(&self) -> f64;
    }
}
```

### 多继承映射

多继承列出所有基类：

```rust
hicc::import_class! {
    #[cpp(class = "ColoredRectangle")]
    class ColoredRectangle: Rectangle, Color {
        // 可以访问所有基类的方法
    }
}
```

### 虚继承

虚继承在 Rust 侧对使用者透明——hicc 在运行时通过 vtable 正确处理菱形继承的单一基类实例。

### 动态分派

对于通过基类指针操作的场景，使用 `Interface<T>` 实现 Rust 侧多态：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    class Shape;

    #[cpp(func = "double total_area_shapes(Shape**, int)")]
    fn total_area(shapes: *mut *mut Shape, n: i32) -> f64;
}
```

## 注意事项

1. **`bases` 解析位置**：Clang AST JSON 中基类信息在节点顶层 `bases` 字段，不在 `inner` 内，cpp2rust-demo 专门处理此特殊结构
2. **纯虚函数识别**：AST 中 `isPure: true` 标志触发 `#[interface]` trait 生成逻辑
3. **多继承方法冲突**：同名方法需要通过强制类型转换或专用包装函数解决
4. **虚继承开销**：虚继承在运行时引入额外间接层，hicc 透明处理，但性能敏感场景需注意
