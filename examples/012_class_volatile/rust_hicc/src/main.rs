hicc::cpp! {
    #include <cstdint>
    #include <iostream>

    class HardwareDevice {
        volatile uint32_t status_reg;
        volatile uint32_t data_reg;
        uint32_t config_reg;
    public:
        HardwareDevice();
        ~HardwareDevice();
        volatile uint32_t readStatus() volatile;
        volatile uint32_t readData() volatile;
        void init();
        void reset();
    };

    HardwareDevice::HardwareDevice() : status_reg(0xA5A5A5A5), data_reg(0), config_reg(0) {}

    HardwareDevice::~HardwareDevice() {}

    volatile uint32_t HardwareDevice::readStatus() volatile {
        return status_reg;
    }

    volatile uint32_t HardwareDevice::readData() volatile {
        return data_reg;
    }

    void HardwareDevice::init() {
        config_reg = 0x00000001;
        status_reg = 0x12345678;
        data_reg = 0;
    }

    void HardwareDevice::reset() {
        status_reg = 0xA5A5A5A5;
        data_reg = 0;
        config_reg = 0;
    }

    HardwareDevice* hardware_device_new() {
        return new HardwareDevice();
    }

    void hardware_device_delete(HardwareDevice* self) {
        delete self;
    }

    uint32_t hardware_device_read_status(HardwareDevice* self) {
        return self->readStatus();
    }

    uint32_t hardware_device_read_data(HardwareDevice* self) {
        return self->readData();
    }

    void hardware_device_init(HardwareDevice* self) {
        self->init();
    }

    void hardware_device_reset(HardwareDevice* self) {
        self->reset();
    }
}

hicc::import_class! {
    #[cpp(class = "HardwareDevice")]
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
    fn hardware_device_new() -> *mut HardwareDevice;

    #[cpp(func = "void hardware_device_delete(HardwareDevice* self)")]
    unsafe fn hardware_device_delete(self_: *mut HardwareDevice);

    #[cpp(func = "uint32_t hardware_device_read_status(HardwareDevice* self)")]
    fn hardware_device_read_status(self_: *mut HardwareDevice) -> u32;

    #[cpp(func = "uint32_t hardware_device_read_data(HardwareDevice* self)")]
    fn hardware_device_read_data(self_: *mut HardwareDevice) -> u32;

    #[cpp(func = "void hardware_device_init(HardwareDevice* self)")]
    fn hardware_device_init(self_: *mut HardwareDevice);

    #[cpp(func = "void hardware_device_reset(HardwareDevice* self)")]
    fn hardware_device_reset(self_: *mut HardwareDevice);
}

fn main() {
    let mut device = hardware_device_new();

    hardware_device_init(&mut device);

    println!("Reading volatile hardware registers (values may change):");
    for i in 0..5 {
        let status = hardware_device_read_status(&mut device);
        let data = hardware_device_read_data(&mut device);
        println!("  Read {}: status=0x{:08x}, data=0x{:08x}", i, status, data);
    }

    hardware_device_reset(&mut device);

    unsafe {
        hardware_device_delete(&device);
    }
    println!("\nRust FFI: volatile qualifier requires volatile pointer in C");
    println!("Note: In C, volatile on the pointed-to object matters for hardware registers");
}
