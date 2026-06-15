use virtual_basic::*;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    println!("=== Virtual Function FFI with hicc ===\n");

    // Create Circle
    let circle = circle_new_with_r(5.0);

    println!("Circle name: {}", decode_cstr(circle.get_name()));
    println!("Circle radius: {}", circle.get_radius());
    println!("Circle area: {:.4}", circle.area());

    println!("\nRust FFI: Virtual functions work through hicc import_class!");
}

