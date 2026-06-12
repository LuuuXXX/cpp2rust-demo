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
    // Derived::area() = value * value = 2.0 * 2.0 = 4.0
    assert!((derived.area() - 4.0).abs() < 1e-10, "Derived::area() 应返回 value² = 4.0");
}

#[test]
fn smoke_base_create() {
    // base_create(0) 创建 Base 实例，area() 返回 0.0
    let base = base_create(0);
    assert!((base.area() - 0.0).abs() < 1e-10, "Base::area() 应返回 0.0");
    let name = unsafe { std::ffi::CStr::from_ptr(base.get_name()) };
    assert!(!name.to_bytes().is_empty(), "Base::getName() 不应返回空字符串");
}

#[test]
fn smoke_base_create_derived_polymorphism() {
    // base_create(1) 创建 Derived(42.0) 以 Base* 返回，验证多态
    let base = base_create(1);
    // Derived::area() = 42.0 * 42.0 = 1764.0
    assert!((base.area() - 1764.0).abs() < 1e-6, "多态：Derived(42.0)::area() 应返回 1764.0");
}
