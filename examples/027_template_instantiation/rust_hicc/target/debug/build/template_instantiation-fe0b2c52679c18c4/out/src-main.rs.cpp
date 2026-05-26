#include <hicc/hicc.hpp>
#line 0 R"(src/main.rs)"
#line 2
#include <iostream>
    #include <vector>
    #include <iomanip>


    template<typename T>
    class Matrix {
        int rows_;
        int cols_;
        std::vector<T> data_;
    public:
        Matrix(int rows, int cols) : rows_(rows), cols_(cols), data_(rows * cols) {}
        int rows() const { return rows_; }
        int cols() const { return cols_; }
        T get(int row, int col) const { return data_[row * cols_ + col]; }
        void set(int row, int col, T value) { data_[row * cols_ + col] = value; }
        void print() const {
            for (int i = 0; i < rows_; i++) {
                for (int j = 0; j < cols_; j++) {
                    std::cout << std::setw(4) << get(i, j);
                }
                std::cout << std::endl;
            }
        }
    };


    class IntMatrix {
        Matrix<int>* impl_;
    public:
        explicit IntMatrix(int rows, int cols) : impl_(new Matrix<int>(rows, cols)) {}
        ~IntMatrix() { delete impl_; }
        int rows() const { return impl_->rows(); }
        int cols() const { return impl_->cols(); }
        int get(int row, int col) const { return impl_->get(row, col); }
        void set(int row, int col, int value) { impl_->set(row, col, value); }
        void print() const { impl_->print(); }
    };

    IntMatrix* intmatrix_new(int rows, int cols) {
        return new IntMatrix(rows, cols);
    }

    void intmatrix_delete(IntMatrix* self_) {
        if (self_) delete self_;
    }


    class DoubleMatrix {
        Matrix<double>* impl_;
    public:
        explicit DoubleMatrix(int rows, int cols) : impl_(new Matrix<double>(rows, cols)) {}
        ~DoubleMatrix() { delete impl_; }
        int rows() const { return impl_->rows(); }
        int cols() const { return impl_->cols(); }
        double get(int row, int col) const { return impl_->get(row, col); }
        void set(int row, int col, double value) { impl_->set(row, col, value); }
        void print() const { impl_->print(); }
    };

    DoubleMatrix* doublematrix_new(int rows, int cols) {
        return new DoubleMatrix(rows, cols);
    }

    void doublematrix_delete(DoubleMatrix* self_) {
        if (self_) delete self_;
    }
#line 72
 struct IntMatrix_72;
#line 72
namespace hicc { template<> struct MethodsType<IntMatrix, void> { typedef IntMatrix_72 methods_type; }; }
#line 90
 struct DoubleMatrix_90;
#line 90
namespace hicc { template<> struct MethodsType<DoubleMatrix, void> { typedef DoubleMatrix_90 methods_type; }; }
#line 72
 struct IntMatrix_72 {
#line 72
typedef IntMatrix Self; typedef void SelfContainer; typedef IntMatrix_72 SelfMethods;
#line 74
static void _hicc_test_74() { int (Self::* _74)() const = &Self::rows; (void)_74; }
#line 74
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::rows));
#line 77
static void _hicc_test_77() { int (Self::* _77)() const = &Self::cols; (void)_77; }
#line 77
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::cols));
#line 80
static void _hicc_test_80() { int (Self::* _80)(int row, int col) const = &Self::get; (void)_80; }
#line 80
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)(int row, int col) const)&Self::get));
#line 83
static void _hicc_test_83() { void (Self::* _83)(int row, int col, int value) = &Self::set; (void)_83; }
#line 83
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(int row, int col, int value))&Self::set));
#line 86
static void _hicc_test_86() { void (Self::* _86)() const = &Self::print; (void)_86; }
#line 86
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)() const)&Self::print));
#line 72
};
#line 90
 struct DoubleMatrix_90 {
#line 90
typedef DoubleMatrix Self; typedef void SelfContainer; typedef DoubleMatrix_90 SelfMethods;
#line 92
static void _hicc_test_92() { int (Self::* _92)() const = &Self::rows; (void)_92; }
#line 92
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::rows));
#line 95
static void _hicc_test_95() { int (Self::* _95)() const = &Self::cols; (void)_95; }
#line 95
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((int (Self::*)() const)&Self::cols));
#line 98
static void _hicc_test_98() { double (Self::* _98)(int row, int col) const = &Self::get; (void)_98; }
#line 98
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((double (Self::*)(int row, int col) const)&Self::get));
#line 101
static void _hicc_test_101() { void (Self::* _101)(int row, int col, double value) = &Self::set; (void)_101; }
#line 101
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)(int row, int col, double value))&Self::set));
#line 104
static void _hicc_test_104() { void (Self::* _104)() const = &Self::print; (void)_104; }
#line 104
EXPORT_MEMBER_METHOD_IN(SelfContainer, ((void (Self::*)() const)&Self::print));
#line 90
};
#line 110
EXPORT_METHODS_BEG(template_instantiation) {
#line 113
static void _hicc_test_113() { IntMatrix* (* _113)(int rows, int cols) = &intmatrix_new; (void)_113; }
#line 113
EXPORT_METHOD_IN(void, ExportMethods, ((IntMatrix* (*)(int rows, int cols))&intmatrix_new));
#line 115
static void _hicc_test_115() { void (* _115)(IntMatrix* self_) = &intmatrix_delete; (void)_115; }
#line 115
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(IntMatrix* self_))&intmatrix_delete));
#line 119
static void _hicc_test_119() { DoubleMatrix* (* _119)(int rows, int cols) = &doublematrix_new; (void)_119; }
#line 119
EXPORT_METHOD_IN(void, ExportMethods, ((DoubleMatrix* (*)(int rows, int cols))&doublematrix_new));
#line 121
static void _hicc_test_121() { void (* _121)(DoubleMatrix* self_) = &doublematrix_delete; (void)_121; }
#line 121
EXPORT_METHOD_IN(void, ExportMethods, ((void (*)(DoubleMatrix* self_))&doublematrix_delete));
#line 110
} EXPORT_METHODS_END();

