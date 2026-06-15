#pragma once

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
