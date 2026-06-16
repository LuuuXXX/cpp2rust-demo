//! 027_template_instantiation 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use template_instantiation::*;

#[test]
fn smoke_int_matrix_dimensions() {
    let m = IntMatrix::new(3, 4);
    assert_eq!(m.rows(), 3, "IntMatrix 行数应为 3");
    assert_eq!(m.cols(), 4, "IntMatrix 列数应为 4");
}

#[test]
fn smoke_int_matrix_get_set() {
    let mut m = IntMatrix::new(2, 2);
    m.set(0, 0, 10);
    m.set(0, 1, 20);
    m.set(1, 0, 30);
    m.set(1, 1, 40);
    assert_eq!(m.get(0, 0), 10);
    assert_eq!(m.get(0, 1), 20);
    assert_eq!(m.get(1, 0), 30);
    assert_eq!(m.get(1, 1), 40);
}

#[test]
fn smoke_double_matrix_dimensions() {
    let m = DoubleMatrix::new(2, 3);
    assert_eq!(m.rows(), 2, "DoubleMatrix 行数应为 2");
    assert_eq!(m.cols(), 3, "DoubleMatrix 列数应为 3");
}

#[test]
fn smoke_double_matrix_get_set() {
    let mut m = DoubleMatrix::new(2, 2);
    m.set(0, 0, 1.1);
    m.set(1, 1, 4.4);
    assert!((m.get(0, 0) - 1.1).abs() < 1e-10);
    assert!((m.get(1, 1) - 4.4).abs() < 1e-10);
}
