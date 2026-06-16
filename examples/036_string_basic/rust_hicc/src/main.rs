use string_basic::*;

fn main() {
    use std::ffi::{CStr, CString};

    println!("=== 036_string_basic - std::string（hicc 直出）===\n");

    let hello = CString::new("hello").expect("CString::new failed");
    let mut s = MyString::new(hello.as_ptr());
    println!("empty={} length={}", s.empty(), s.length());

    let suffix = CString::new(", world").expect("CString::new failed");
    s.append(suffix.as_ptr());
    let content = unsafe { CStr::from_ptr(s.c_str()).to_string_lossy().into_owned() };
    println!("after append={} length={}", content, s.length());

    println!("at(1)={} at(99)={}", s.at(1) as u8 as char, s.at(99));

    let compare = CString::new("hello").expect("CString::new failed");
    println!("compare hello={}", s.compare(compare.as_ptr()));

    let world = CString::new("world").expect("CString::new failed");
    let missing = CString::new("missing").expect("CString::new failed");
    println!("find world={} find missing={}", s.find(world.as_ptr()), s.find(missing.as_ptr()));

    s.to_upper();
    let upper = unsafe { CStr::from_ptr(s.c_str()).to_string_lossy().into_owned() };
    println!("to_upper={}", upper);

    println!("\nRust FFI: hicc 直接绑定持有 std::string 的类，析构由 Rust Drop 自动完成");
}
