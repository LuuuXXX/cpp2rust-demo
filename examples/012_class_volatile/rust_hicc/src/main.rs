hicc::cpp! {
    #include <iostream>
    #include <cstdint>

    #include "class_volatile.h"
}

hicc::import_class! {
    #[cpp(class = "HardwareDevice", destroy = "hardware_device_delete")]
    class HardwareDevice {
        #[cpp(method = "uint32_t readStatus()")]
        fn read_status(&mut self) -> u32;

        #[cpp(method = "uint32_t readData()")]
        fn read_data(&mut self) -> u32;

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
}

fn main() {
    use hicc::AbiClass;
    let mut device = hardware_device_new();

    device.init();

    println!("Reading volatile hardware registers (values may change):");
    for i in 0..5 {
        let status = device.read_status();
        let data = device.read_data();
        println!("  Read {}: status=0x{:08x}, data=0x{:08x}", i, status, data);
    }

    device.reset();

    println!("\nRust FFI: volatile methods are directly accessible as struct methods");
    println!("Note: volatile qualifier on C++ methods is handled transparently by hicc");
}
