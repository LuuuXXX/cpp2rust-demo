use inheritance_single::*;

fn main() {
    // Create Animal
    let animal_name = "Generic Animal\0";
    let animal = unsafe { animal_new(animal_name.as_ptr() as *const i8) };

    println!("Animal name: {}", decode_cstr(animal.get_name()));
    animal.speak();

    println!();

    // Create Dog
    let dog_name = "Buddy\0";
    let dog = unsafe { dog_new(dog_name.as_ptr() as *const i8) };

    println!("Dog name: {}", decode_cstr(dog.get_name()));
    dog.speak();  // Call inherited speak method
    dog.bark();   // Call Dog's own bark method

    println!("\nRust FFI: Single inheritance with hicc pattern");
}

fn decode_cstr(ptr: *const i8) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { std::ffi::CStr::from_ptr(ptr) }
        .to_string_lossy()
        .to_string()
}
