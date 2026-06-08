hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <string>

    #include "virtual_override.h"
}

hicc::import_class! {
    #[cpp(class = "Base", destroy = "base_delete")]
    pub class Base {
        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Derived", destroy = "derived_delete")]
    pub class Derived {
        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;

        #[cpp(method = "double getValue() const")]
        fn get_value(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "virtual_override"]

    class Base;
    class Derived;

    #[cpp(func = "Base* base_create(int)")]
    fn base_create(type_: i32) -> Base;

    #[cpp(func = "Derived* derived_new(double)")]
    fn derived_new(value: f64) -> Derived;

    // 虚函数通过 C ABI 包装调用，避免 macOS ARM64 vtable 兼容问题
    #[cpp(func = "double base_area(Base*)")]
    fn base_area(self_: *mut Base) -> f64;
}

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    use hicc::AbiClass;

    println!("=== Virtual Override FFI with hicc ===\n");
    println!("The 'override' keyword explicitly marks method overriding in C++\n");

    // Create Base
    let mut base = unsafe { base_create(0) };

    // Create Derived (through base_create returning Base*)
    let mut derived = unsafe { base_create(1) };

    println!("--- Calling through Base pointer ---");
    println!("Name: {}", decode_cstr(base.get_name()));
    println!("Area: {:.4}", base_area(&base.as_mut_ptr()));

    println!();
    println!("--- Calling through Derived (as Base*) ---");
    println!("Name: {}", decode_cstr(derived.get_name()));
    println!("Area: {:.4}", base_area(&derived.as_mut_ptr()));

    println!();
    println!("override ensures Derived::area() is called not Base::area()");
    println!("This is polymorphism: same interface, different implementations\n");

    println!("Rust FFI: override keyword works correctly through hicc!");
}

