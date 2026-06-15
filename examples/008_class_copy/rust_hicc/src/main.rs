use class_copy::*;

fn main() {
    let mut buf1 = buffer_new_with_sz(5);
    println!("buf1 size: {}", buf1.get_size());

    for i in 0..5 {
        buf1.set(i, (i + 1) * 10);
    }

    print!("buf1 values: ");
    for i in 0..5 {
        print!("{} ", buf1.get(i));
    }
    println!();

    buf1.set(0, 999);
    println!("After modifying buf1[0] = 999:");
    println!("buf1[0] = {}", buf1.get(0));

    println!("\nRust FFI: Buffer class works!");
}
