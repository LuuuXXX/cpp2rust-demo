hicc::cpp! {
    #include <iostream>
    #include <cstdint>

    #include "class_volatile.h"
}

hicc::import_class! {
    #[cpp(class = "HardwareDevice")]
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

    #[cpp(func = "std::unique_ptr<HardwareDevice> hicc::make_unique<HardwareDevice>()")]
    pub fn hardware_device_new() -> HardwareDevice;
}
