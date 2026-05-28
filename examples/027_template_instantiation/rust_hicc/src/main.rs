hicc::cpp! {
    #include <iostream>
    #include <vector>
    #include <iomanip>

    class IntMatrix {
        Matrix<int>* impl_;
    public:
        IntMatrix(int rows, int cols) : impl_(new Matrix<int>(rows, cols)) {}
        ~IntMatrix() { delete impl_; }
        int rows() const { return impl_->rows(); }
        int cols() const { return impl_->cols(); }
        int get(int row, int col) const { return impl_->get(row, col); }
        void set(int row, int col, int value) { impl_->set(row, col, value); }
        void print() const { impl_->print(); }
    };

    class DoubleMatrix {
        Matrix<double>* impl_;
    public:
        DoubleMatrix(int rows, int cols) : impl_(new Matrix<double>(rows, cols)) {}
        ~DoubleMatrix() { delete impl_; }
        int rows() const { return impl_->rows(); }
        int cols() const { return impl_->cols(); }
        double get(int row, int col) const { return impl_->get(row, col); }
        void set(int row, int col, double value) { impl_->set(row, col, value); }
        void print() const { impl_->print(); }
    };

    IntMatrix* intmatrix_new(int rows, int cols) {
        return new IntMatrix(rows, cols);
    }

    void intmatrix_delete(IntMatrix* self) {
        if (self) delete self;
    }

    DoubleMatrix* doublematrix_new(int rows, int cols) {
        return new DoubleMatrix(rows, cols);
    }

    void doublematrix_delete(DoubleMatrix* self) {
        if (self) delete self;
    }
}

hicc::import_lib! {
    #![link_name = "template_instantiation"]

    class IntMatrix;
    class DoubleMatrix;

    #[cpp(class = "IntMatrix")]
    class IntMatrix {
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

        #[cpp(func = "IntMatrix* intmatrix_new(int, int)")]
        fn new(rows: i32, cols: i32) -> *mut IntMatrix;

        #[cpp(func = "void intmatrix_delete(IntMatrix* self)")]
        unsafe fn delete(self_: *mut IntMatrix);
    }

    #[cpp(class = "DoubleMatrix")]
    class DoubleMatrix {
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

        #[cpp(func = "DoubleMatrix* doublematrix_new(int, int)")]
        fn new(rows: i32, cols: i32) -> *mut DoubleMatrix;

        #[cpp(func = "void doublematrix_delete(DoubleMatrix* self)")]
        unsafe fn delete(self_: *mut DoubleMatrix);
    }
}

fn main() {
    println!("=== 027_template_instantiation - 模板显式实例化 ===\n");

    // IntMatrix
    let mut im = intmatrix_new(3, 3);
    im.set(0, 0, 1);
    im.set(0, 1, 2);
    im.set(0, 2, 3);
    im.set(1, 0, 4);
    im.set(1, 1, 5);
    im.set(1, 2, 6);
    im.set(2, 0, 7);
    im.set(2, 1, 8);
    im.set(2, 2, 9);
    im.print();
    unsafe { intmatrix_delete(&im) };

    println!();

    // DoubleMatrix
    let mut dm = doublematrix_new(2, 2);
    dm.set(0, 0, 1.1);
    dm.set(0, 1, 2.2);
    dm.set(1, 0, 3.3);
    dm.set(1, 1, 4.4);
    dm.print();
    unsafe { doublematrix_delete(&dm) };

    println!("\nRust FFI: 显式实例化将模板绑定到具体类型");
    println!("extern template 声明可在库中预实例化");
    println!("Matrix<int> -> IntMatrix");
    println!("Matrix<double> -> DoubleMatrix");
}


