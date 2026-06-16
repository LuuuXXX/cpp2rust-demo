//! 023_typeid_rtti 冒烟测试：经基类引用的 typeid 取回动态类型名（RTTI）。

use typeid_rtti::*;
use std::ffi::CStr;

fn type_name(p: *const i8) -> String {
    // 安全性：指针来自 typeid(...).name()，指向静态生命周期的 C 字符串。
    unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
}

#[test]
fn areas() {
    assert!((Circle::new(2.0).area() - 12.566_370_6).abs() < 1e-4);
    assert_eq!(Rectangle::new(3.0, 4.0).area(), 12.0);
    assert_eq!(Triangle::new(6.0, 2.0).area(), 6.0);
}

#[test]
fn rtti_runtime_type_names() {
    // typeid 名称在不同平台/ABI 下格式不同（Itanium 含 "6Circle"，MSVC 含 "Circle"），
    // 但都包含类名，故用 contains 断言以保证跨平台稳定。
    assert!(type_name(Circle::new(1.0).runtime_type_name()).contains("Circle"));
    assert!(type_name(Rectangle::new(1.0, 1.0).runtime_type_name()).contains("Rectangle"));
    assert!(type_name(Triangle::new(1.0, 1.0).runtime_type_name()).contains("Triangle"));
}
