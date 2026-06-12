hicc::cpp! {
    #include <iostream>
    #include <cstdint>

    #include "class_volatile.h"
}

hicc::import_class! {
    #[cpp(class = "HardwareDevice", destroy = "hardware_device_delete")]
    pub class HardwareDevice {
        #[cpp(method = "void init()")]
        pub fn init(&mut self);

        #[cpp(method = "void reset()")]
        pub fn reset(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "class_volatile"]

    class HardwareDevice;

    #[cpp(func = "HardwareDevice* hardware_device_new()")]
    pub fn hardware_device_new() -> HardwareDevice;

    #[cpp(func = "uint32_t hardware_device_read_status(volatile HardwareDevice*)")]
    pub unsafe fn hardware_device_read_status(self_: *mut HardwareDevice) -> u32;

    #[cpp(func = "uint32_t hardware_device_read_data(volatile HardwareDevice*)")]
    pub unsafe fn hardware_device_read_data(self_: *mut HardwareDevice) -> u32;
}
