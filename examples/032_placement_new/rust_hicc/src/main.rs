use placement_new::*;

fn main() {
    println!("=== 032_placement_new - 定位 new（hicc 直出）===\n");

    let mut buf = Buffer::new(64);
    println!("capacity={}", buf.capacity());
    println!(
        "construct_at(0,42)={} value_at(0)={} size={}",
        buf.construct_at(0, 42),
        buf.value_at(0),
        buf.size()
    );
    println!(
        "construct_at(8,7)={} value_at(8)={}",
        buf.construct_at(8, 7),
        buf.value_at(8)
    );

    println!();

    let mut arr = ObjectArray::new(3);
    println!("count={} element_size={}", arr.count(), arr.element_size());
    for i in 0..arr.count() {
        arr.emplace(i, (i + 1) * 10);
    }
    println!("at(0)={} at(1)={} at(2)={}", arr.at(0), arr.at(1), arr.at(2));

    println!("\nRust FFI: hicc 绑定在预分配存储中用 placement new 构造对象的类");
}
