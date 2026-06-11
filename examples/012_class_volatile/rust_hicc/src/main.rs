use class_volatile::*;

fn main() {
    use hicc::AbiClass;
    let mut device = hardware_device_new();

    device.init();

    println!("Reading volatile hardware registers (values may change):");
    for i in 0..5 {
        let status = unsafe { hardware_device_read_status(&device.as_mut_ptr()) };
        let data = unsafe { hardware_device_read_data(&device.as_mut_ptr()) };
        println!("  Read {}: status=0x{:08x}, data=0x{:08x}", i, status, data);
    }

    println!();
    println!("Rust FFI: volatile qualifier requires volatile pointer in C");
    println!("Note: In C, volatile on the pointed-to object matters for hardware registers");
}
