# 示例 03：基础类

## 特性概述

本示例展示 C++ **类（class）** 的核心特性：构造函数、析构函数、访问控制（`public`/`private`/`protected`）、成员方法（const/非const）、嵌套类。类是 C++ 面向对象编程的基础，hicc 通过 `import_class!` 宏提供对应的 Rust 映射。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| 构造函数 | 默认构造、参数化构造、拷贝构造 |
| 析构函数 | 资源释放 |
| 访问控制 | `private` 数据成员，`public` 方法接口 |
| `const` 方法 | 不修改对象状态的方法 |
| 非 `const` 方法 | 修改对象状态的方法 |
| 嵌套类 | 定义在外部类内部的类 |
| `struct` | 默认 `public` 的聚合类型 |

### 代码结构

```cpp
class Rectangle {
private:
    double width, height;
public:
    Rectangle();                          // 默认构造
    Rectangle(double w, double h);        // 参数化构造
    Rectangle(const Rectangle& other);    // 拷贝构造
    ~Rectangle();                         // 析构函数
    double area() const;                  // const 方法
    void resize(double w, double h);      // 非 const 方法
    double get_width() const;
};

struct Point { double x, y; };

class Outer {
    class Inner { ... };                  // 嵌套类
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
| `CXXRecordDecl` | class/struct 声明 |
| `CXXConstructorDecl` | 构造函数（含 `isDefaulted` 标志） |
| `CXXDestructorDecl` | 析构函数 |
| `CXXMethodDecl` | 普通成员方法（`const` 方法含 `isConst: true`） |
| `FieldDecl` | 成员变量 |
| `AccessSpecDecl` | 访问控制说明符 |

AST 片段示例（构造函数）：

```json
{
  "kind": "CXXConstructorDecl",
  "name": "Rectangle",
  "type": { "qualType": "void (double, double)" },
  "inner": [
    { "kind": "ParmVarDecl", "name": "w", "type": { "qualType": "double" } },
    { "kind": "ParmVarDecl", "name": "h", "type": { "qualType": "double" } }
  ]
}
```

## hicc 处理方式

### 类映射：`import_class!`

C++ 类通过 `hicc::import_class!` 映射为 Rust struct：

```rust
hicc::import_class! {
    #[cpp(class = "Rectangle")]
    class Rectangle {
        // const 方法 → &self
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "double perimeter() const")]
        fn perimeter(&self) -> f64;

        #[cpp(method = "double get_width() const")]
        fn get_width(&self) -> f64;

        // 非 const 方法 → &mut self
        #[cpp(method = "void resize(double, double)")]
        fn resize(&mut self, w: f64, h: f64);
    }
}
```

### 构造函数 → `import_lib!`

构造函数（对象创建）通过 `import_lib!` 声明工厂函数：

```rust
hicc::import_lib! {
    #![link_name = "example"]

    class Rectangle;

    // 默认构造 → make_unique 工厂
    #[cpp(func = "std::unique_ptr<Rectangle> std::make_unique<Rectangle>()")]
    fn rectangle_new() -> Rectangle;

    // 参数化构造
    #[cpp(func = "std::unique_ptr<Rectangle> std::make_unique<Rectangle, double, double>(double&&, double&&)")]
    fn rectangle_new_wh(w: f64, h: f64) -> Rectangle;
}
```

### `const` 方法 vs 非 `const` 方法

| C++ | Rust | 说明 |
|-----|------|------|
| `double area() const` | `fn area(&self) -> f64` | 只读借用 |
| `void resize(double, double)` | `fn resize(&mut self, ...)` | 可变借用 |

cpp2rust-demo 通过 AST 中 `CXXMethodDecl` 的 `isConst` 字段自动判断并生成对应的 `&self` 或 `&mut self`。

### 嵌套类处理

嵌套类在 AST 中以独立的 `CXXRecordDecl` 出现（带完整限定名），可以单独映射：

```rust
hicc::import_class! {
    #[cpp(class = "Outer::Inner")]
    class OuterInner {
        #[cpp(method = "int get_value() const")]
        fn get_value(&self) -> i32;
    }
}
```

## 注意事项

1. **对象所有权**：通过 `import_lib!` 工厂函数创建的对象由 Rust 所有权系统管理，离开作用域时自动调用 C++ 析构函数
2. **`const` 方法映射**：hicc 通过 `&self` vs `&mut self` 精确反映 C++ 的 `const` 语义
3. **私有成员**：C++ 的 `private` 字段不暴露给 Rust，只通过公开方法访问
4. **拷贝构造**：C++ 拷贝构造需要通过专门的工厂函数或 `AbiClass::write` 方法来实现
