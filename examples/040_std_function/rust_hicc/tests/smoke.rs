//! 040_std_function 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use std_function::*;
use hicc::AbiClass;

#[test]
fn smoke_callback_wrapper_double() {
    let mut wrapper = callback_wrapper_new_double();
    let result = wrapper.invoke(5);
    assert_eq!(result, 10, "callback_wrapper_new_double invoke(5) 应返回 10");
}

#[test]
fn smoke_callback_wrapper_double_multiple() {
    let mut wrapper = callback_wrapper_new_double();
    assert_eq!(wrapper.invoke(7), 14, "invoke(7) 应返回 14");
    assert_eq!(wrapper.invoke(0), 0, "invoke(0) 应返回 0");
    assert_eq!(wrapper.invoke(-3), -6, "invoke(-3) 应返回 -6");
}

#[test]
fn smoke_processor() {
    let mut processor = processor_new();
    unsafe { processor_set_double(&processor.as_mut_ptr()); }
    let result = processor.process(10);
    assert_eq!(result, 20, "processor(设置 double 后) process(10) 应返回 20");
}

#[test]
fn smoke_multi_callback() {
    let mut mc = multi_callback_new();
    unsafe {
        multi_callback_add_double(&mc.as_mut_ptr());
        multi_callback_add_triple(&mc.as_mut_ptr());
    }
    // invoke_all 只是打印输出，不崩溃即可
    mc.invoke_all(4);
}

#[test]
fn smoke_async_processor_initial() {
    let ap = async_processor_new();
    assert!(!ap.is_cancelled(), "AsyncProcessor 初始状态应未取消");
}

#[test]
fn smoke_async_processor_cancel() {
    let mut ap = async_processor_new();
    assert!(!ap.is_cancelled(), "初始应未取消");
    ap.cancel();
    assert!(ap.is_cancelled(), "cancel() 后应为已取消");
}
