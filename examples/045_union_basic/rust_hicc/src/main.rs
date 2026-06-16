use union_basic::*;

fn main() {
    use std::ffi::{CStr, CString};

    println!("=== 045_union_basic - union（hicc 直出）===\n");

    let mut v = Variant::new();
    v.set_int(42);
    println!("variant int type={} value={}", v.get_type(), v.get_int());
    v.set_float(2.5);
    println!("variant float type={} value={}", v.get_type(), v.get_float());
    let hi = CString::new("hi").expect("CString::new failed");
    v.set_string(hi.as_ptr());
    let s = unsafe { CStr::from_ptr(v.get_string()).to_string_lossy().into_owned() };
    println!("variant string type={} value={}", v.get_type(), s);

    let mut u = IntFloatUnion::new();
    u.set_int(7);
    println!("union int={}", u.get_int());
    u.set_float(1.5);
    println!("union float={}", u.get_float());

    println!("\nRust FFI: hicc 直接绑定持有 union 的类，析构由 Rust Drop 自动完成");
}
