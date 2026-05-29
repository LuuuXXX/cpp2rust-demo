#pragma once

#ifdef __cplusplus
extern "C" {
#endif

// 显式实例化声明示例
// 使用 extern template 声明，模板实例化在另外的 .cpp 文件中

// Matrix<int> 声明
class IntMatrix;

IntMatrix* intmatrix_new(int rows, int cols);
void intmatrix_delete(IntMatrix* self);

int intmatrix_get(IntMatrix* self, int row, int col);
void intmatrix_set(IntMatrix* self, int row, int col, int value);

int intmatrix_rows(IntMatrix* self);
int intmatrix_cols(IntMatrix* self);

void intmatrix_print(IntMatrix* self);

// Matrix<double> 声明
class DoubleMatrix;

DoubleMatrix* doublematrix_new(int rows, int cols);
void doublematrix_delete(DoubleMatrix* self);

double doublematrix_get(DoubleMatrix* self, int row, int col);
void doublematrix_set(DoubleMatrix* self, int row, int col, double value);

int doublematrix_rows(DoubleMatrix* self);
int doublematrix_cols(DoubleMatrix* self);

void doublematrix_print(DoubleMatrix* self);

#ifdef __cplusplus
}

// Full class definition - for hicc code generation
#include <vector>
#include <iostream>
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

#endif
