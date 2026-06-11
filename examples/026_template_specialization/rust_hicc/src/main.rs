use template_specialization::*;

fn main() {
    println!("=== 026_template_specialization - 模板偏特化 ===\n");

    // IntHolder (通用版本)
    let ih = intholder_new(42);
    let ih_desc = unsafe { std::ffi::CStr::from_ptr(ih.describe()) };
    println!("{}", ih_desc.to_string_lossy());
    println!("  get(): {}", ih.get());

    println!();

    // DoubleHolder (通用版本)
    let dh = doubleholder_new(3.14159);
    let dh_desc = unsafe { std::ffi::CStr::from_ptr(dh.describe()) };
    println!("{}", dh_desc.to_string_lossy());
    println!("  get(): {:.5}", dh.get());

    println!();

    // StringHolder (char* 特化版本)
    let s = std::ffi::CString::new("Hello, World!").expect("CString::new failed");
    let sh = unsafe { stringholder_new(s.as_ptr()) };
    let sh_desc = unsafe { std::ffi::CStr::from_ptr(sh.describe()) };
    println!("{}", sh_desc.to_string_lossy());
    let sh_val = unsafe { std::ffi::CStr::from_ptr(sh.get()) };
    println!("  get(): {}", sh_val.to_string_lossy());

    println!("\nRust FFI: 每个模板特化是独立的结构");
    println!("通用版本: IntHolder, DoubleHolder");
    println!("偏特化: StringHolder (处理 char*)");
}

