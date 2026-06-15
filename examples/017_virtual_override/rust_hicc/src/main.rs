use virtual_override::*;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    println!("=== Virtual Override FFI with hicc ===\n");
    println!("The 'override' keyword explicitly marks method overriding in C++\n");

    let base_name = std::ffi::CString::new("Base").expect("CString::new failed");
    let base = unsafe { base_new_with_n(base_name.as_ptr()) };

    let derived = derived_new_with_v(3.14);

    println!("--- Calling through Base ---");
    println!("Name: {}", decode_cstr(base.get_name()));
    println!("Area: {:.4}", base.area());

    println!();
    println!("--- Calling through Derived (override) ---");
    println!("Name: {}", decode_cstr(derived.get_name()));
    println!("Area: {:.4}", derived.area());
    println!("Value: {:.4}", derived.get_value());

    println!();
    println!("override ensures Derived::area() is called not Base::area()");
    println!("This is polymorphism: same interface, different implementations\n");

    println!("Rust FFI: override keyword works correctly through hicc!");
}

