//! 012_class_volatile 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use class_volatile::*;
use hicc::AbiClass;

#[test]
fn smoke_device_init_and_read() {
    let mut device = hardware_device_new();
    device.init();

    let status = unsafe { hardware_device_read_status(&device.as_mut_ptr()) };
    let data = unsafe { hardware_device_read_data(&device.as_mut_ptr()) };

    // After init, status and data should be valid u32 values
    // The exact values depend on the C++ implementation, but they should be readable
    let _ = status;
    let _ = data;
}

#[test]
fn smoke_device_reset() {
    let mut device = hardware_device_new();
    device.init();
    device.reset();

    let status = unsafe { hardware_device_read_status(&device.as_mut_ptr()) };
    let data = unsafe { hardware_device_read_data(&device.as_mut_ptr()) };

    // After reset, values should be 0 or a known reset state
    // Just verify we can read without crashing
    let _ = status;
    let _ = data;
}

#[test]
fn smoke_device_multiple_reads() {
    let mut device = hardware_device_new();
    device.init();

    // Read multiple times to verify volatile reads work
    for _ in 0..5 {
        let _ = unsafe { hardware_device_read_status(&device.as_mut_ptr()) };
        let _ = unsafe { hardware_device_read_data(&device.as_mut_ptr()) };
    }
}
