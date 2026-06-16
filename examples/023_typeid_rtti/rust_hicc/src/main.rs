use typeid_rtti::*;
use std::ffi::CStr;

fn type_name<'a>(p: *const i8) -> &'a str {
    // 安全性：指针来自 typeid(...).name()，指向静态生命周期的 C 字符串。
    unsafe { CStr::from_ptr(p) }.to_str().unwrap_or("?")
}

fn main() {
    let c = Circle::new(2.0);
    let r = Rectangle::new(3.0, 4.0);
    let t = Triangle::new(6.0, 2.0);
    println!("circle area={:.4} typeid={}", c.area(), type_name(c.runtime_type_name()));
    println!("rect   area={:.4} typeid={}", r.area(), type_name(r.runtime_type_name()));
    println!("tri    area={:.4} typeid={}", t.area(), type_name(t.runtime_type_name()));
    println!("--- end main ---");
}
