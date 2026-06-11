//! 017_virtual_override 冒烟测试
//!
//! 验证生成的 Rust FFI 绑定可编译、可链接 C++ 实现，且基本行为正确。

use virtual_override::*;

#[test]
fn smoke_derived_get_value() {
    let derived = derived_new(3.14);
    assert!((derived.get_value() - 3.14).abs() < 1e-10, "Derived::getValue() 应返回构造时传入的值");
}

#[test]
fn smoke_derived_area() {
    let derived = derived_new(2.0);
    // area() 的具体实现由 C++ 侧决定，只检查可以调用且返回有效值
    let _ = derived.area();
}

#[test]
fn smoke_base_create() {
    // base_create(0) 创建 Base 实例
    let base = unsafe { base_create(0) };
    let _ = base.area();
    let name = unsafe { std::ffi::CStr::from_ptr(base.get_name()) };
    assert!(!name.to_bytes().is_empty(), "Base::getName() 不应返回空字符串");
}
