use default_args::*;

fn main() {
    let name = b"World\0".as_ptr() as *const i8;

    unsafe {
        let result = greet(name, 1);
        println!("greet(\"World\", 1) returned: {}", result);
    }

    fn greet_with_default(name: *const i8) -> i32 {
        unsafe { greet(name, 1) }
    }

    let result = greet_with_default(name);
    println!("greet_with_default(\"World\") returned: {}", result);

    println!("\nRust FFI: Default args simulated in Rust!");
}
