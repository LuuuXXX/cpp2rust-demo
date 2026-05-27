hicc::cpp! {
    #include "default_args.h"
}

hicc::import_lib! {
    #![link_name = "default_args"]

    #[cpp(func = "int greet(const char*, int)")]
    unsafe fn greet(name: *const i8, times: i32) -> i32;
}

fn main() {
    let name = b"World\0".as_ptr() as *const i8;

    // 显式传递所有参数
    unsafe {
        let result = greet(name, 1);
        println!("greet(\"World\", 1) returned: {}", result);
    }

    // Rust 层面模拟默认参数
    fn greet_with_default(name: *const i8) -> i32 {
        unsafe { greet(name, 1) }  // 默认 times = 1
    }

    let result = greet_with_default(name);
    println!("greet_with_default(\"World\") returned: {}", result);

    println!("\nRust FFI: Default args simulated in Rust!");
}

