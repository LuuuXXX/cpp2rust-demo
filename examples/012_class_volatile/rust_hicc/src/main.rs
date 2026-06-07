hicc::cpp! {
    #include <iostream>
    #include <cstdint>

    #include "class_volatile.h"
}

hicc::import_class! {
    #[cpp(class = "HardwareDevice", destroy = "hardware_device_delete")]
    pub class HardwareDevice {
        #[cpp(method = "void init()")]
        fn init(&mut self);

        #[cpp(method = "void reset()")]
        fn reset(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "class_volatile"]

    class HardwareDevice;

    #[cpp(func = "HardwareDevice* hardware_device_new()")]
    fn hardware_device_new() -> HardwareDevice;

    #[cpp(func = "uint32_t hardware_device_read_status(volatile HardwareDevice*)")]
    unsafe fn hardware_device_read_status(self_: *mut HardwareDevice) -> u32;

    #[cpp(func = "uint32_t hardware_device_read_data(volatile HardwareDevice*)")]
    unsafe fn hardware_device_read_data(self_: *mut HardwareDevice) -> u32;
}

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
