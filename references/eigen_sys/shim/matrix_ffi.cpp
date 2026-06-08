// Eigen Matrix extern-C 包装层（matrix_ffi.cpp）
//
// 封装 Eigen::Matrix3f（3×3 float 矩阵）为 extern "C" opaque-handle 接口，
// 验证工具在密集模板实例化（ClassTemplateSpecialization）场景下
// 能正确生成 import_lib! FFI 绑定。
//
// 使用系统安装的 libeigen3-dev（Ubuntu: apt-get install libeigen3-dev）。
// 头文件路径：/usr/include/eigen3/Eigen/Dense

#include <Eigen/Dense>
#include <cstdlib>

extern "C" {

/// 创建 3×3 float 零矩阵
void* eigen_matrix3f_create() {
    return new Eigen::Matrix3f(Eigen::Matrix3f::Zero());
}

/// 释放矩阵
void eigen_matrix3f_destroy(void* m) {
    delete static_cast<Eigen::Matrix3f*>(m);
}

/// 获取元素 (row, col)
float eigen_matrix3f_get(void* m, int row, int col) {
    return (*static_cast<Eigen::Matrix3f*>(m))(row, col);
}

/// 设置元素 (row, col)
void eigen_matrix3f_set(void* m, int row, int col, float val) {
    (*static_cast<Eigen::Matrix3f*>(m))(row, col) = val;
}

/// 返回矩阵行数（固定为 3）
int eigen_matrix3f_rows(void* m) {
    return static_cast<Eigen::Matrix3f*>(m)->rows();
}

/// 返回矩阵列数（固定为 3）
int eigen_matrix3f_cols(void* m) {
    return static_cast<Eigen::Matrix3f*>(m)->cols();
}

/// 返回 Frobenius 范数
float eigen_matrix3f_norm(void* m) {
    return static_cast<Eigen::Matrix3f*>(m)->norm();
}

/// 矩阵乘法：result = a * b（result 需预先分配）
void eigen_matrix3f_mul(void* a, void* b, void* result) {
    *static_cast<Eigen::Matrix3f*>(result) =
        (*static_cast<Eigen::Matrix3f*>(a)) *
        (*static_cast<Eigen::Matrix3f*>(b));
}

} // extern "C"
