use typeid_rtti::*;
use std::ffi::CStr;

fn type_name<'a>(p: *const i8) -> &'a str {
    // 安全性：指针来自 typeid(...).name()，指向静态生命周期的 C 字符串。
    unsafe { CStr::from_ptr(p) }.to_str().unwrap_or("?")
}

fn dynamic_type(p: *const i8) -> &'static str {
    // typeid 名称格式因 ABI 而异（Itanium：N14...6CircleE，MSVC：class typeid_rtti_ns::Circle），
    // 但都包含简单类名，故归一化为简单类名以保证跨平台稳定输出。
    let name = type_name(p);
    for cls in ["Circle", "Rectangle", "Triangle"] {
        if name.contains(cls) {
            return cls;
        }
    }
    "?"
}

fn main() {
    let c = Circle::new(2.0);
    let r = Rectangle::new(3.0, 4.0);
    let t = Triangle::new(6.0, 2.0);
    println!("circle area={:.4} typeid={}", c.area(), dynamic_type(c.runtime_type_name()));
    println!("rect   area={:.4} typeid={}", r.area(), dynamic_type(r.runtime_type_name()));
    println!("tri    area={:.4} typeid={}", t.area(), dynamic_type(t.runtime_type_name()));
    println!("--- end main ---");
}
