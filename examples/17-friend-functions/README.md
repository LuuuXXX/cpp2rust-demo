# 示例 17：友元函数与友元类

## 特性概述

本示例展示 C++ 的**友元**机制，包括友元函数、友元类以及运算符的友元重载。友元打破了 C++ 的封装边界，允许外部函数或类访问 `private`/`protected` 成员。在 hicc 中，友元声明的全局函数作为普通全局函数处理。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| 友元函数 | `friend double dot(const Vector2D&, const Vector2D&)` |
| 友元运算符重载 | `friend Vector2D operator+(const Vector2D&, const Vector2D&)` |
| 友元类 | `friend class Vector2DFactory` |

### 代码结构

```cpp
class Vector2D {
    double x, y;  // private

    // 友元函数声明
    friend Vector2D operator+(const Vector2D& a, const Vector2D& b);
    friend double dot(const Vector2D& a, const Vector2D& b);
    friend class Vector2DFactory;  // 友元类
};

// 友元函数定义（全局函数）
Vector2D operator+(const Vector2D& a, const Vector2D& b) {
    return Vector2D(a.x + b.x, a.y + b.y);
}

double dot(const Vector2D& a, const Vector2D& b) {
    return a.x * b.x + a.y * b.y;
}

// 友元类
class Vector2DFactory {
    static Vector2D create_from_xy(double x, double y);  // 可访问 Vector2D 私有成员
};
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 节点：

| AST 节点 | 含义 |
|----------|------|
| `FriendDecl` | 友元声明（在类内部） |
| `FunctionDecl`（全局） | 友元函数的实际定义（在类外部） |
| `CXXRecordDecl` 含 `FriendDecl` | 有友元的类 |

友元函数在 AST 中出现两次：
1. 在类内作为 `FriendDecl` 节点（声明）
2. 在翻译单元根部作为 `FunctionDecl` 节点（定义）

## hicc 处理方式

### 友元函数 → 全局函数

友元函数虽然在类内声明，但实际上是**全局函数**，通过 `import_lib!` 映射：

```rust
hicc::import_class! {
    #[cpp(class = "Vector2D")]
    class Vector2D {
        #[cpp(method = "double get_x() const")]
        fn get_x(&self) -> f64;

        #[cpp(method = "double get_y() const")]
        fn get_y(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "example"]

    class Vector2D;

    // 友元函数作为全局函数绑定
    #[cpp(func = "Vector2D operator+(const Vector2D&, const Vector2D&)")]
    fn vector_add(a: &Vector2D, b: &Vector2D) -> Vector2D;

    #[cpp(func = "Vector2D operator-(const Vector2D&, const Vector2D&)")]
    fn vector_sub(a: &Vector2D, b: &Vector2D) -> Vector2D;

    #[cpp(func = "double dot(const Vector2D&, const Vector2D&)")]
    fn dot(a: &Vector2D, b: &Vector2D) -> f64;
}
```

### 友元运算符重载 → 运算符垫片

友元运算符与成员运算符的处理方式相同，cpp2rust-demo 为其生成运算符垫片：

```rust
// 通过垫片绑定友元运算符
hicc::import_lib! {
    #![link_name = "example"]

    class Vector2D;

    // operator+ 的垫片
    #[cpp(func = "Vector2D shim_add_Vector2D_Vector2D(const Vector2D&, const Vector2D&)")]
    fn vector2d_add(a: &Vector2D, b: &Vector2D) -> Vector2D;
}

// 实现 Rust Add trait
use std::ops::Add;
impl Add for Vector2D {
    type Output = Vector2D;
    fn add(self, rhs: Vector2D) -> Vector2D {
        vector2d_add(&self, &rhs)
    }
}
```

### 友元类映射

友元类本身是普通的 C++ 类，其特殊之处在于可以访问另一个类的私有成员。在 Rust 侧，只需正常映射两个类：

```rust
hicc::import_class! {
    #[cpp(class = "Vector2DFactory")]
    class Vector2DFactory {
        #[cpp(method = "Vector2D Vector2DFactory::create_from_xy(double, double) const")]
        fn create_from_xy(&self, x: f64, y: f64) -> Vector2D;
    }
}
```

### 非成员函数组织

对于较多的全局辅助函数，可以在 Rust 侧通过模块组织：

```rust
pub mod vector2d_ops {
    use super::*;

    pub fn add(a: &Vector2D, b: &Vector2D) -> Vector2D { vector_add(a, b) }
    pub fn sub(a: &Vector2D, b: &Vector2D) -> Vector2D { vector_sub(a, b) }
    pub fn dot(a: &Vector2D, b: &Vector2D) -> f64 { dot(a, b) }
}
```

## 注意事项

1. **友元不破坏封装的原则**：友元是 C++ 设计决策，Rust 侧无法访问对应的私有字段（hicc 不暴露内部状态）
2. **友元函数的 AST 位置**：cpp2rust-demo 优先查找翻译单元根部的 `FunctionDecl`，不依赖 `FriendDecl` 来生成绑定
3. **友元运算符与成员运算符**：两者在 FFI 层面无区别，都通过运算符垫片处理
4. **友元类的限制**：友元类关系只是 C++ 访问控制，对 Rust 侧完全透明，无需特殊处理
