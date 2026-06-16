use explicit_ctor::*;

fn main() {
    let a = Widget::new(42);    // 由 int 构造
    let b = Widget::new_2(3.9); // 由 double 显式构造（截断为 3）
    println!("a.get_value()={}", a.get_value());
    println!("b.get_value()={}", b.get_value());
    println!("--- end main ---");
}
