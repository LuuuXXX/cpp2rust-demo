use class_volatile::*;

fn main() {
    let mut device = hardware_device_new();

    device.init();
    device.reset();

    println!();
    println!("Rust FFI: volatile qualifier requires volatile pointer in C");
    println!("Note: In C, volatile on the pointed-to object matters for hardware registers");
}
