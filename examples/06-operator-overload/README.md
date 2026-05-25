# 示例 06：运算符重载

## 特性概述

本示例展示 C++ 的**运算符重载**，包括算术运算符、比较运算符、下标运算符、函数调用运算符以及类型转换运算符。hicc 通过运算符垫片（operator shims）机制将这些运算符映射为 Rust 方法。

## C++ 特性说明

| 特性 | 说明 |
|------|------|
| 算术运算符 | `operator+`、`operator-`、`operator*`、`operator/` |
| 复合赋值运算符 | `operator+=`、`operator-=` 等 |
| 比较运算符 | `operator==`、`operator<` 等 |
| 下标运算符 | `operator[]` |
| 函数调用运算符 | `operator()` |
| 类型转换运算符 | `operator double()` 等 |
| 前缀/后缀自增 | `operator++()` / `operator++(int)` |

### 代码结构（Complex 类）

```cpp
class Complex {
    Complex operator+(const Complex& other) const;
    Complex operator-(const Complex& other) const;
    Complex operator*(const Complex& other) const;
    Complex operator/(const Complex& other) const;
    Complex& operator+=(const Complex& other);
    bool operator==(const Complex& other) const;
    bool operator!=(const Complex& other) const;
    operator double() const;  // 类型转换运算符
    double get_real() const;
    double get_imag() const;
};
```

## AST JSON 结构要点

运行以下命令生成 `ast.json`：

```bash
clang++ -Xclang -ast-dump=json -fsyntax-only main.cpp > ast.json
```

关键 AST 节点：

| AST 节点 / 属性 | 含义 |
|-----------------|------|
| `CXXMethodDecl.name = "operator+"` | 加法运算符重载 |
| `CXXMethodDecl.name = "operator[]"` | 下标运算符 |
| `CXXMethodDecl.name = "operator()"` | 函数调用运算符 |
| `CXXConversionDecl` | 类型转换运算符（`operator double()` 等） |
| `CXXMethodDecl.name = "operator++"` | 自增运算符（前缀/后缀由参数数量区分） |

AST 片段示例：

```json
{
  "kind": "CXXMethodDecl",
  "name": "operator+",
  "type": { "qualType": "Complex (const Complex &) const" }
}
```

## hicc 处理方式

### 运算符垫片机制

由于 Rust trait（`Add`、`Sub` 等）与 C++ 运算符签名不完全对应，cpp2rust-demo 会在 `meta/` 目录下生成运算符垫片文件：

- `meta/operator_shims.hpp`：C++ 侧垫片函数声明
- `rust/src/free/shim_ops.rs`：Rust 侧 `import_lib!` 绑定

垫片函数命名规则：

| C++ 运算符 | 垫片函数名 |
|------------|-----------|
| `operator+` | `shim_add_Complex_Complex` |
| `operator-` | `shim_sub_Complex_Complex` |
| `operator*` | `shim_mul_Complex_Complex` |
| `operator+=` | `shim_add_assign_Complex_Complex` |
| `operator==` | `shim_eq_Complex_Complex` |
| `operator[]` | `shim_index_Complex_usize` |

### Rust 绑定示例

```rust
hicc::import_class! {
    #[cpp(class = "Complex")]
    class Complex {
        #[cpp(method = "double get_real() const")]
        fn get_real(&self) -> f64;

        #[cpp(method = "double get_imag() const")]
        fn get_imag(&self) -> f64;

        // 类型转换
        #[cpp(method = "double Complex::operator double() const")]
        fn to_double(&self) -> f64;
    }
}

// 运算符通过垫片函数调用
hicc::import_lib! {
    #![link_name = "example"]

    class Complex;

    #[cpp(func = "Complex shim_add_Complex_Complex(const Complex&, const Complex&)")]
    fn complex_add(a: &Complex, b: &Complex) -> Complex;

    #[cpp(func = "bool shim_eq_Complex_Complex(const Complex&, const Complex&)")]
    fn complex_eq(a: &Complex, b: &Complex) -> bool;
}
```

### 实现 Rust 标准运算符 Trait

垫片绑定后，可以为 Rust 类型实现标准 trait：

```rust
use std::ops::Add;

impl Add for Complex {
    type Output = Complex;
    fn add(self, rhs: Complex) -> Complex {
        complex_add(&self, &rhs)
    }
}
```

### 下标运算符

```rust
hicc::import_lib! {
    #![link_name = "example"]

    #[cpp(func = "double& MyVector::operator[](size_t)")]
    fn vector_index_mut(v: &mut MyVector, idx: usize) -> &mut f64;
}
```

### 函数调用运算符

```rust
hicc::import_class! {
    #[cpp(class = "Multiplier")]
    class Multiplier {
        // operator()(int x) 映射为方法调用
        #[cpp(method = "int Multiplier::operator()(int) const")]
        fn call(&self, x: i32) -> i32;
    }
}
```

## 注意事项

1. **垫片自动生成**：cpp2rust-demo 自动检测 AST 中的 `operator*` 方法并生成对应垫片，开发者无需手动编写
2. **复合赋值运算符**：`+=`、`-=` 等通过独立垫片函数处理，返回类型统一为 `*mut T`
3. **跨 TU 去重**：垫片名基于 `(shim_name, param_sig)` 组合去重，确保多翻译单元下名称唯一
4. **类型转换运算符**：`operator double()` 等通过 `CXXConversionDecl` 节点检测，映射为普通方法
5. **`operator<<`（流运算符）**：通常为友元函数，参见示例 17
