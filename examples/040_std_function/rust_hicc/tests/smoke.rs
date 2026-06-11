//! 040_std_function 冒烟测试
//!
//! std::function 回调经 class wrapper 暴露；验证回调注入后的计算结果。

use std_function::*;
use hicc::AbiClass;

#[test]
fn smoke_callback_wrapper_double() {
    let mut wrapper = callback_wrapper_new_double();
    assert_eq!(wrapper.invoke(5), 10, "double 回调 invoke(5) 应为 10");
    assert_eq!(wrapper.invoke(7), 14, "double 回调 invoke(7) 应为 14");
}

#[test]
fn smoke_processor_set_double() {
    let mut processor = processor_new();
    unsafe { processor_set_double(&processor.as_mut_ptr()) };
    assert_eq!(processor.process(10), 20, "注入 double 回调后 process(10) 应为 20");
}

#[test]
fn smoke_async_processor_cancel() {
    let mut ap = async_processor_new();
    assert!(!ap.is_cancelled(), "初始未取消");
    ap.cancel();
    assert!(ap.is_cancelled(), "cancel 后应为已取消");
}
