#include "template_instantiation.h"
#include <iostream>
#include <vector>
#include <iomanip>

// C-compatible wrapper functions
IntMatrix* intmatrix_new(int rows, int cols) {
    return new IntMatrix(rows, cols);
}

void intmatrix_delete(IntMatrix* self) {
    if (self) delete self;
}

int intmatrix_get(IntMatrix* self, int row, int col) {
    return self->get(row, col);
}

void intmatrix_set(IntMatrix* self, int row, int col, int value) {
    self->set(row, col, value);
}

int intmatrix_rows(IntMatrix* self) {
    return self->rows();
}

int intmatrix_cols(IntMatrix* self) {
    return self->cols();
}

void intmatrix_print(IntMatrix* self) {
    self->print();
}

// Matrix template implementation
template<typename T>
Matrix<T>::Matrix(int rows, int cols) : rows_(rows), cols_(cols), data_(rows * cols) {}

template<typename T>
int Matrix<T>::rows() const { return rows_; }

template<typename T>
int Matrix<T>::cols() const { return cols_; }

template<typename T>
T Matrix<T>::get(int row, int col) const { return data_[row * cols_ + col]; }

template<typename T>
void Matrix<T>::set(int row, int col, T value) { data_[row * cols_ + col] = value; }

template<typename T>
void Matrix<T>::print() const {
    for (int i = 0; i < rows_; i++) {
        for (int j = 0; j < cols_; j++) {
            std::cout << std::setw(4) << get(i, j);
        }
        std::cout << std::endl;
    }
}

// IntMatrix implementation
IntMatrix::IntMatrix(int rows, int cols) : impl_(new Matrix<int>(rows, cols)) {}
IntMatrix::~IntMatrix() { delete impl_; }
int IntMatrix::rows() const { return impl_->rows(); }
int IntMatrix::cols() const { return impl_->cols(); }
int IntMatrix::get(int row, int col) const { return impl_->get(row, col); }
void IntMatrix::set(int row, int col, int value) { impl_->set(row, col, value); }
void IntMatrix::print() const { impl_->print(); }

DoubleMatrix* doublematrix_new(int rows, int cols) {
    return new DoubleMatrix(rows, cols);
}

void doublematrix_delete(DoubleMatrix* self) {
    if (self) delete self;
}

double doublematrix_get(DoubleMatrix* self, int row, int col) {
    return self->get(row, col);
}

void doublematrix_set(DoubleMatrix* self, int row, int col, double value) {
    self->set(row, col, value);
}

int doublematrix_rows(DoubleMatrix* self) {
    return self->rows();
}

int doublematrix_cols(DoubleMatrix* self) {
    return self->cols();
}

void doublematrix_print(DoubleMatrix* self) {
    self->print();
}

// DoubleMatrix implementation
DoubleMatrix::DoubleMatrix(int rows, int cols) : impl_(new Matrix<double>(rows, cols)) {}
DoubleMatrix::~DoubleMatrix() { delete impl_; }
int DoubleMatrix::rows() const { return impl_->rows(); }
int DoubleMatrix::cols() const { return impl_->cols(); }
double DoubleMatrix::get(int row, int col) const { return impl_->get(row, col); }
void DoubleMatrix::set(int row, int col, double value) { impl_->set(row, col, value); }
void DoubleMatrix::print() const { impl_->print(); }
