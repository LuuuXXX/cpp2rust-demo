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
template<typename T>
class Matrix {
    int rows_;
    int cols_;
    std::vector<T> data_;
public:
    Matrix(int rows, int cols);
    int rows() const;
    int cols() const;
    T get(int row, int col) const;
    void set(int row, int col, T value);
    void print() const;
};

class IntMatrix {
    Matrix<int>* impl_;
public:
    explicit IntMatrix(int rows, int cols);
    ~IntMatrix();
    int rows() const;
    int cols() const;
    int get(int row, int col) const;
    void set(int row, int col, int value);
    void print() const;
};

class DoubleMatrix {
    Matrix<double>* impl_;
public:
    explicit DoubleMatrix(int rows, int cols);
    ~DoubleMatrix();
    int rows() const;
    int cols() const;
    double get(int row, int col) const;
    void set(int row, int col, double value);
    void print() const;
};

#endif
