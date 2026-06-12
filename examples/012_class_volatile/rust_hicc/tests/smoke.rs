//! 012_class_volatile 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。
//! C++ 侧：构造时 status_reg=0xA5A5A5A5, data_reg=0；
//!         init() 后 status_reg=0x12345678, data_reg=0；
//!         reset() 后 status_reg=0xA5A5A5A5, data_reg=0。

use class_volatile::*;
use hicc::AbiClass;

#[test]
fn smoke_device_initial_state() {
    let mut device = hardware_device_new();
    let status = unsafe { hardware_device_read_status(&device.as_mut_ptr()) };
    let data = unsafe { hardware_device_read_data(&device.as_mut_ptr()) };
    assert_eq!(status, 0xA5A5A5A5, "构造后 status_reg 应为 0xA5A5A5A5");
    assert_eq!(data, 0u32, "构造后 data_reg 应为 0");
}

#[test]
fn smoke_device_init_and_read() {
    let mut device = hardware_device_new();
    device.init();

    let status = unsafe { hardware_device_read_status(&device.as_mut_ptr()) };
    let data = unsafe { hardware_device_read_data(&device.as_mut_ptr()) };
    assert_eq!(status, 0x12345678, "init() 后 status_reg 应为 0x12345678");
    assert_eq!(data, 0u32, "init() 后 data_reg 应为 0");
}

#[test]
fn smoke_device_reset() {
    let mut device = hardware_device_new();
    device.init();
    device.reset();

    let status = unsafe { hardware_device_read_status(&device.as_mut_ptr()) };
    let data = unsafe { hardware_device_read_data(&device.as_mut_ptr()) };
    assert_eq!(status, 0xA5A5A5A5, "reset() 后 status_reg 应恢复为 0xA5A5A5A5");
    assert_eq!(data, 0u32, "reset() 后 data_reg 应为 0");
}

#[test]
fn smoke_device_multiple_reads() {
    let mut device = hardware_device_new();
    device.init();

    for _ in 0..5 {
        let status = unsafe { hardware_device_read_status(&device.as_mut_ptr()) };
        let data = unsafe { hardware_device_read_data(&device.as_mut_ptr()) };
        assert_eq!(status, 0x12345678, "多次读取 status_reg 应保持一致");
        assert_eq!(data, 0u32, "多次读取 data_reg 应保持一致");
    }
}
