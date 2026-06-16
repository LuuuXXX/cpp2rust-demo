//! 027_template_instantiation: 模板实例化（命名空间类模板 Matrix<T> + 具体实例化类）。
//!
//! C++ 类模板 `Matrix<T>` 在使用点按具体类型实例化（`Matrix<int>`、`Matrix<double>`）。
//! 本示例将每个实例化暴露为 idiomatic 命名空间类（`IntMatrix` / `DoubleMatrix`），
//! hicc 直出按普通类绑定方法与 `make_unique` 工厂，无需任何 extern-C shim。

hicc::cpp! {
    #include "template_instantiation.h"
}

hicc::import_class! {
    #[cpp(class = "template_instantiation_ns::IntMatrix")]
    pub class IntMatrix {
        #[cpp(method = "int rows() const")]
        pub fn rows(&self) -> i32;

        #[cpp(method = "int cols() const")]
        pub fn cols(&self) -> i32;

        #[cpp(method = "int get(int row, int col) const")]
        pub fn get(&self, row: i32, col: i32) -> i32;

        #[cpp(method = "void set(int row, int col, int value)")]
        pub fn set(&mut self, row: i32, col: i32, value: i32);

        #[cpp(method = "void print() const")]
        pub fn print(&self);

        pub fn new(rows: i32, cols: i32) -> Self { int_matrix_new(rows, cols) }
    }
}

hicc::import_class! {
    #[cpp(class = "template_instantiation_ns::DoubleMatrix")]
    pub class DoubleMatrix {
        #[cpp(method = "int rows() const")]
        pub fn rows(&self) -> i32;

        #[cpp(method = "int cols() const")]
        pub fn cols(&self) -> i32;

        #[cpp(method = "double get(int row, int col) const")]
        pub fn get(&self, row: i32, col: i32) -> f64;

        #[cpp(method = "void set(int row, int col, double value)")]
        pub fn set(&mut self, row: i32, col: i32, value: f64);

        #[cpp(method = "void print() const")]
        pub fn print(&self);

        pub fn new(rows: i32, cols: i32) -> Self { double_matrix_new(rows, cols) }
    }
}

hicc::import_lib! {
    #![link_name = "template_instantiation"]

    #[cpp(func = "std::unique_ptr<template_instantiation_ns::IntMatrix> hicc::make_unique<template_instantiation_ns::IntMatrix, int, int>(int&&, int&&)")]
    pub fn int_matrix_new(rows: i32, cols: i32) -> IntMatrix;

    #[cpp(func = "std::unique_ptr<template_instantiation_ns::DoubleMatrix> hicc::make_unique<template_instantiation_ns::DoubleMatrix, int, int>(int&&, int&&)")]
    pub fn double_matrix_new(rows: i32, cols: i32) -> DoubleMatrix;
}
