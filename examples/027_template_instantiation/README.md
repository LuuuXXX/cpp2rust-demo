# 027_template_instantiation - 模板显式实例化

## C++ 特性

本示例展示 C++ 显式实例化声明（`extern template`）和显式实例化定义的使用。

## C++ 代码

### template_instantiation.cpp

```cpp
// 模板类
template<typename T>
class Matrix {
    int rows_, cols_;
    std::vector<T> data_;
public:
    Matrix(int rows, int cols);
    T get(int row, int col) const;
    void set(int row, int col, T value);
};

// 显式实例化
template class Matrix<int>;    // 在编译时实例化
template class Matrix<double>;
```

## Rust FFI 代码

```rust
hicc::cpp! {
    #include <iostream>
    #include <vector>
    #include <iomanip>

    #include "template_instantiation.h"
}

hicc::import_class! {
    #[cpp(class = "IntMatrix", destroy = "intmatrix_delete")]
    pub class IntMatrix {
        #[cpp(method = "int rows() const")]
        fn rows(&self) -> i32;

        #[cpp(method = "int cols() const")]
        fn cols(&self) -> i32;

        #[cpp(method = "int get(int row, int col) const")]
        fn get(&self, row: i32, col: i32) -> i32;

        #[cpp(method = "void set(int row, int col, int value)")]
        fn set(&mut self, row: i32, col: i32, value: i32);

        #[cpp(method = "void print() const")]
        fn print(&self);
    }
}

hicc::import_class! {
    #[cpp(class = "DoubleMatrix", destroy = "doublematrix_delete")]
    pub class DoubleMatrix {
        #[cpp(method = "int rows() const")]
        fn rows(&self) -> i32;

        #[cpp(method = "int cols() const")]
        fn cols(&self) -> i32;

        #[cpp(method = "double get(int row, int col) const")]
        fn get(&self, row: i32, col: i32) -> f64;

        #[cpp(method = "void set(int row, int col, double value)")]
        fn set(&mut self, row: i32, col: i32, value: f64);

        #[cpp(method = "void print() const")]
        fn print(&self);
    }
}

hicc::import_lib! {
    #![link_name = "template_instantiation"]

    class IntMatrix;
    class DoubleMatrix;

    #[cpp(func = "IntMatrix* intmatrix_new(int, int)")]
    fn intmatrix_new(rows: i32, cols: i32) -> IntMatrix;

    #[cpp(func = "DoubleMatrix* doublematrix_new(int, int)")]
    fn doublematrix_new(rows: i32, cols: i32) -> DoubleMatrix;
}
```
## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 显式实例化 | `template class Matrix<int>` | 手动创建 wrapper |
| extern 声明 | `extern template` 避免重复实例化 | 不需要 |
| 库集成 | 可在库中预实例化 | 导出时必须实例化 |

## extern template 的作用

```cpp
// lib.h
extern template class Matrix<int>;  // 声明在其他地方实例化

// lib.cpp
template class Matrix<int>;  // 实际实例化
```

- 避免在多个翻译单元中重复生成相同实例化代码
- 减少编译时间和二进制大小
- 在 FFI 中，我们可以预先实例化需要的类型

## 运行结果

```
=== 027_template_instantiation - 模板显式实例化 ===

   1   2   3
   4   5   6
   7   8   9

 1.1 2.2
 3.3 4.4

Rust FFI: 显式实例化将模板绑定到具体类型
extern template 声明可在库中预实例化
Matrix<int> -> IntMatrix
Matrix<double> -> DoubleMatrix
```

## 总结

- 显式实例化是模板 FFI 的标准方法
- 在库中预先实例化常用类型
- FFI 层负责将模板实例化导出为独立函数