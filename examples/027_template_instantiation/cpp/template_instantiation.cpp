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
