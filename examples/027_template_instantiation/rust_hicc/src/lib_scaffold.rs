hicc::cpp! {
    #include <iostream>
    #include <vector>
    #include <iomanip>

    #include "template_instantiation.h"

    std::unique_ptr<IntMatrix> _cpp2rust_make_unique_int_matrix_2(int rows, int cols) { return std::make_unique<IntMatrix>(rows, cols); }
    std::unique_ptr<DoubleMatrix> _cpp2rust_make_unique_double_matrix_2(int rows, int cols) { return std::make_unique<DoubleMatrix>(rows, cols); }
}

hicc::import_class! {
    #[cpp(class = "IntMatrix")]
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
    }
}

hicc::import_class! {
    #[cpp(class = "DoubleMatrix")]
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
    }
}

hicc::import_lib! {
    #![link_name = "template_instantiation"]

    class IntMatrix;
    class DoubleMatrix;

    #[cpp(func = "std::unique_ptr<IntMatrix> _cpp2rust_make_unique_int_matrix_2(int, int)")]
    pub fn int_matrix_new_2(rows: i32, cols: i32) -> IntMatrix;

    #[cpp(func = "std::unique_ptr<DoubleMatrix> _cpp2rust_make_unique_double_matrix_2(int, int)")]
    pub fn double_matrix_new_2(rows: i32, cols: i32) -> DoubleMatrix;
}
