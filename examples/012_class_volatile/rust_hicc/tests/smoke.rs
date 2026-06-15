//! 012_class_volatile 冒烟测试：volatile 寄存器读取与配置。

use class_volatile::*;

#[test]
fn initial_register_values() {
    let dev = HardwareDevice::new();
    assert_eq!(dev.read_status(), 0xA5A5A5A5);
    assert_eq!(dev.read_data(), 0);
}

#[test]
fn init_and_reset_registers() {
    let mut dev = HardwareDevice::new();
    dev.init();
    assert_eq!(dev.read_status(), 0x12345678, "init 后 status 应更新");
    assert_eq!(dev.read_data(), 0);

    dev.reset();
    assert_eq!(dev.read_status(), 0xA5A5A5A5, "reset 后回到初始值");
    assert_eq!(dev.read_data(), 0);
}
