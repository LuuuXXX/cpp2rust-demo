use virtual_override::*;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    println!("=== Virtual Override FFI with hicc ===\n");
    println!("The 'override' keyword explicitly marks method overriding in C++\n");

    // Create Base
    let base = base_create(0);

    // Create Derived (through base_create returning Base*)
    let derived = base_create(1);

    println!("--- Calling through Base pointer ---");
    println!("Name: {}", decode_cstr(base.get_name()));
    println!("Area: {:.4}", base.area());

    println!();
    println!("--- Calling through Derived (as Base*) ---");
    println!("Name: {}", decode_cstr(derived.get_name()));
    println!("Area: {:.4}", derived.area());

    println!();
    println!("override ensures Derived::area() is called not Base::area()");
    println!("This is polymorphism: same interface, different implementations\n");

    println!("Rust FFI: override keyword works correctly through hicc!");
}

