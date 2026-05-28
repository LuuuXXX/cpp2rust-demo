hicc::cpp! {
    #include <iostream>
    #include <cstdint>

    class HardwareDevice {
        volatile uint32_t status_reg;
        volatile uint32_t data_reg;
        uint32_t config_reg;
    public:
        HardwareDevice() : status_reg(0xA5A5A5A5), data_reg(0), config_reg(0) {}
        ~HardwareDevice() {}
        uint32_t readStatus() {
    return status_reg;
}
        uint32_t readData() {
    return data_reg;
}
        void init() {
    config_reg = 0x00000001;
    status_reg = 0x12345678;
    data_reg = 0;
}
        void reset() {
    status_reg = 0xA5A5A5A5;
    data_reg = 0;
    config_reg = 0;
}
    };

    HardwareDevice* hardware_device_new() {
        return new HardwareDevice();
    }

    void hardware_device_delete(HardwareDevice* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "HardwareDevice")]
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
    fn hardware_device_new() -> *mut HardwareDevice;

    #[cpp(func = "void hardware_device_delete(HardwareDevice* self)")]
    unsafe fn hardware_device_delete(self_: *mut HardwareDevice);
}

fn main() {
    let mut device = hardware_device_new();

    device.init();

    println!("Reading volatile hardware registers (values may change):");
    for i in 0..5 {
        let status = device.read_status();
        let data = device.read_data();
        println!("  Read {}: status=0x{:08x}, data=0x{:08x}", i, status, data);
    }

    device.reset();

    unsafe {
        hardware_device_delete(&device);
    }
    println!("\nRust FFI: volatile qualifier requires volatile pointer in C");
    println!("Note: In C, volatile on the pointed-to object matters for hardware registers");
}



