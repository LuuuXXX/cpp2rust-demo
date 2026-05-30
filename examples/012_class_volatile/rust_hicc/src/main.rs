hicc::cpp! {
    #include <iostream>
    #include <cstdint>

    #include "class_volatile.h"
}

hicc::import_class! {
    #[cpp(class = "HardwareDevice", destroy = "hardware_device_delete")]
    class HardwareDevice {
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

    println!("Device initialized. Volatile methods (readStatus/readData) require");
    println!("manual C++ shim wrappers as hicc does not support volatile method pointers.");

    device.reset();

    println!("Device reset. Volatile hardware register access requires custom wrappers.");
}
