use class_volatile::*;

fn main() {
    let mut dev = HardwareDevice::new();
    println!("status=0x{:08x} data=0x{:08x}", dev.read_status(), dev.read_data());

    dev.init();
    println!("after init status=0x{:08x} data=0x{:08x}", dev.read_status(), dev.read_data());

    dev.reset();
    println!("after reset status=0x{:08x}", dev.read_status());
}
