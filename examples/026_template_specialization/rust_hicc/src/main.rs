use template_specialization::*;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    println!("=== 026_template_specialization - 模板偏特化 ===\n");

    // IntHolder (通用版本)
    let ih = int_holder_new_with_value(42);
    println!("{}", decode_cstr(ih.describe()));
    println!("  get(): {}", ih.get());

    println!();

    // DoubleHolder (通用版本)
    let dh = double_holder_new_with_value(3.14159);
    println!("{}", decode_cstr(dh.describe()));
    println!("  get(): {:.5}", dh.get());

    println!();

    // StringHolder (char* 特化版本)
    let s = std::ffi::CString::new("Hello, World!").expect("CString::new failed");
    let sh = unsafe { string_holder_new_with_value(s.as_ptr()) };
    println!("{}", decode_cstr(sh.describe()));
    println!("  get(): {}", decode_cstr(sh.get()));

    println!("\nRust FFI: 每个模板特化是独立的结构");
    println!("通用版本: IntHolder, DoubleHolder");
    println!("偏特化: StringHolder (处理 char*)");
}
