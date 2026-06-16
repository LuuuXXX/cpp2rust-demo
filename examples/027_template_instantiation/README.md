# 027_template_instantiation - 模板实例化（hicc 直出，去 shim）

## C++ 特性

本示例展示 C++ **模板实例化**的 FFI 处理方式。类模板 `Matrix<T>` 在使用点按具体类型
实例化（`Matrix<int>`、`Matrix<double>`）。每个实例化暴露为 idiomatic 命名空间类
（`IntMatrix` / `DoubleMatrix`），hicc 直出按普通类绑定方法与 `make_unique` 工厂，
无需任何 extern-C shim。

## C++ 代码

### template_instantiation.h

```cpp
namespace template_instantiation_ns {

// 类模板
template <typename T>
class Matrix {
    int rows_, cols_;
    std::vector<T> data_;
public:
    Matrix(int rows, int cols) : rows_(rows), cols_(cols), data_(rows * cols) {}
    int rows() const { return rows_; }
    int cols() const { return cols_; }
    T get(int row, int col) const { return data_[row * cols_ + col]; }
    void set(int row, int col, T value) { data_[row * cols_ + col] = value; }
    void print() const { /* ... */ }
};

// 显式实例化为具体类
class IntMatrix    { Matrix<int>    impl_; /* ... */ };
class DoubleMatrix { Matrix<double> impl_; /* ... */ };

} // namespace template_instantiation_ns
```

## Rust FFI 代码

hicc 直出无需 extern-C shim，直接绑定具体类与 `make_unique` 工厂：

```rust
hicc::cpp! {
    #include "template_instantiation.h"
}

hicc::import_class! {
    #[cpp(class = "template_instantiation_ns::IntMatrix")]
    pub class IntMatrix {
        #[cpp(method = "int rows() const")]
        pub fn rows(&self) -> i32;
        #[cpp(method = "int get(int row, int col) const")]
        pub fn get(&self, row: i32, col: i32) -> i32;
        #[cpp(method = "void set(int row, int col, int value)")]
        pub fn set(&mut self, row: i32, col: i32, value: i32);
        // cols() / print() 略

        pub fn new(rows: i32, cols: i32) -> Self { int_matrix_new(rows, cols) }
    }
}

// DoubleMatrix 同理（Matrix<double>）

hicc::import_lib! {
    #![link_name = "template_instantiation"]

    #[cpp(func = "std::unique_ptr<template_instantiation_ns::IntMatrix> hicc::make_unique<template_instantiation_ns::IntMatrix, int, int>(int&&, int&&)")]
    pub fn int_matrix_new(rows: i32, cols: i32) -> IntMatrix;
    // double_matrix_new 同理
}
```

## FFI 对比分析

| 方面 | C++ | Rust FFI |
|------|-----|----------|
| 模板定义 | `template<T> class Matrix` | 不可直接导出 |
| 模板实例化 | `Matrix<int>` / `Matrix<double>` | 暴露为 `IntMatrix` / `DoubleMatrix` |
| 构造 | `IntMatrix(rows, cols)` | `IntMatrix::new(i32, i32)`（`make_unique`） |
| 内存管理 | RAII | hicc `unique_ptr` 自动析构 |

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

## 冒烟测试

本示例包含集成冒烟测试（`rust_hicc/tests/smoke.rs`），验证生成的 Rust FFI 绑定可编译、
链接并正确调用。

### 测试用例

| 测试函数 | 验证内容 |
|---------|---------|
| `smoke_int_matrix_dimensions` | `IntMatrix` 行列数 |
| `smoke_int_matrix_get_set` | `IntMatrix` 读写元素 |
| `smoke_double_matrix_dimensions` | `DoubleMatrix` 行列数 |
| `smoke_double_matrix_get_set` | `DoubleMatrix` 读写元素 |

### 运行方式

```bash
cd examples/027_template_instantiation/rust_hicc
cargo test --test smoke
```

### 各平台支持

| 平台 | 状态 | 备注 |
|------|------|------|
| Linux (Ubuntu) | ✅ | CI `l-smoke` job 已覆盖 |
| macOS | ✅ | 支持 |
| Windows MinGW | ✅ | 支持 |

## 总结

- C++ 类模板无法直接 FFI 导出（模板本身没有可链接符号）
- 策略：将每个具体实例化暴露为 idiomatic 命名空间类
- hicc 直出按普通类绑定方法与 `make_unique` 工厂，无需 extern-C shim
- `Matrix<int>` → `IntMatrix`，`Matrix<double>` → `DoubleMatrix`
