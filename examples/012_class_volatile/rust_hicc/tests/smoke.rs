//! 012_class_volatile 冒烟测试
//!
//! volatile 限定通过 volatile 指针的 C shim 暴露；验证 init/reset 后寄存器值。

use class_volatile::*;
use hicc::AbiClass;

#[test]
fn smoke_device_init_status() {
    let mut device = hardware_device_new();
    device.init();
    let status = unsafe { hardware_device_read_status(&device.as_mut_ptr()) };
    assert_eq!(status, 0x12345678, "init 后 status 寄存器应为 0x12345678");
    let data = unsafe { hardware_device_read_data(&device.as_mut_ptr()) };
    assert_eq!(data, 0, "init 后 data 寄存器应为 0");
}

#[test]
fn smoke_device_reset_status() {
    let mut device = hardware_device_new();
    device.init();
    device.reset();
    let status = unsafe { hardware_device_read_status(&device.as_mut_ptr()) };
    assert_eq!(status, 0xA5A5A5A5, "reset 后 status 寄存器应为 0xA5A5A5A5");
}
