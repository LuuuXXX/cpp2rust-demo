use class_copy::*;

fn main() {
    use hicc::AbiClass;

    // Create buffer
    let mut buf1 = buffer_new_with_size(5);
    println!("buf1 size: {}", buf1.get_size());

    // Set values
    for i in 0..5 {
        buf1.set(i, (i + 1) * 10);
    }

    // Get values
    print!("buf1 values: ");
    for i in 0..5 {
        print!("{} ", buf1.get(i));
    }
    println!();

    // Copy constructor
    let buf2 = buffer_new_copy(&buf1.as_ptr());
    println!("buf2 created by copy");
    println!("buf2 size: {}", buf2.get_size());

    print!("buf2 values: ");
    for i in 0..5 {
        print!("{} ", buf2.get(i));
    }
    println!();

    // Modifying original does not affect copy
    buf1.set(0, 999);
    println!("After modifying buf1[0] = 999:");
    println!("buf1[0] = {}", buf1.get(0));
    println!("buf2[0] = {} (unchanged)", buf2.get(0));

    println!("\nRust FFI: Copy constructor pattern works!");
}
