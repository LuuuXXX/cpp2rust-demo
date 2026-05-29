hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <cstdlib>
    #include <cstdio>

    #include "template_specialization.h"
}

hicc::import_class! {
    #[cpp(class = "IntHolder", destroy = "intholder_delete")]
    class IntHolder {
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;

        #[cpp(method = "const char* describe() const")]
        fn describe(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "DoubleHolder", destroy = "doubleholder_delete")]
    class DoubleHolder {
        #[cpp(method = "double get() const")]
        fn get(&self) -> f64;

        #[cpp(method = "const char* describe() const")]
        fn describe(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "StringHolder", destroy = "stringholder_delete")]
    class StringHolder {
        #[cpp(method = "const char* get() const")]
        fn get(&self) -> *const i8;

        #[cpp(method = "const char* describe() const")]
        fn describe(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "template_specialization"]

    class IntHolder;
    class DoubleHolder;
    class StringHolder;

    #[cpp(func = "IntHolder* intholder_new(int)")]
    fn intholder_new(value: i32) -> IntHolder;

    #[cpp(func = "DoubleHolder* doubleholder_new(double)")]
    fn doubleholder_new(value: f64) -> DoubleHolder;

    #[cpp(func = "StringHolder* stringholder_new(const char*)")]
    unsafe fn stringholder_new(value: *const i8) -> StringHolder;
}

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

