use template_instantiation::*;

fn main() {
    println!("=== 027_template_instantiation - 模板显式实例化 ===\n");

    // IntMatrix
    let mut im = int_matrix_new_2(3, 3);
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

    println!();

    // DoubleMatrix
    let mut dm = double_matrix_new_2(2, 2);
    dm.set(0, 0, 1.1);
    dm.set(0, 1, 2.2);
    dm.set(1, 0, 3.3);
    dm.set(1, 1, 4.4);
    dm.print();

    println!("\nRust FFI: 显式实例化将模板绑定到具体类型");
    println!("extern template 声明可在库中预实例化");
    println!("Matrix<int> -> IntMatrix");
    println!("Matrix<double> -> DoubleMatrix");
}
