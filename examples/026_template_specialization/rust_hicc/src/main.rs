use template_specialization::*;

fn main() {
    println!("=== 026_template_specialization - 模板特化 ===\n");

    // IntHolder：通用模板 ValueHolder<int>
    let ih = IntHolder::new(42);
    let ih_desc = unsafe { std::ffi::CStr::from_ptr(ih.describe()) };
    println!("{}", ih_desc.to_string_lossy());
    println!("  get(): {}", ih.get());

    println!();

    // DoubleHolder：通用模板 ValueHolder<double>
    let dh = DoubleHolder::new(3.14159);
    let dh_desc = unsafe { std::ffi::CStr::from_ptr(dh.describe()) };
    println!("{}", dh_desc.to_string_lossy());
    println!("  get(): {:.5}", dh.get());

    println!();

    // StringHolder：全特化 ValueHolder<std::string>
    let s = std::ffi::CString::new("Hello, World!").expect("CString::new failed");
    let sh = StringHolder::new(s.as_ptr());
    let sh_desc = unsafe { std::ffi::CStr::from_ptr(sh.describe()) };
    println!("{}", sh_desc.to_string_lossy());
    let sh_val = unsafe { std::ffi::CStr::from_ptr(sh.get()) };
    println!("  get(): {}", sh_val.to_string_lossy());

    println!("\nRust FFI: 通用模板与特化各自暴露为独立的具体类");
    println!("通用版本: IntHolder, DoubleHolder（ValueHolder<T>）");
    println!("全特化:  StringHolder（ValueHolder<std::string>）");
}
