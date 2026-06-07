hicc::cpp! {
    #include <cstddef>
    #include <iostream>
    #include <stdexcept>
    #include <utility>

    #include "noexcept_basic.h"
}

hicc::import_class! {
    #[cpp(class = "NoexceptMover", destroy = "noexcept_mover_delete")]
    pub class NoexceptMover {
        #[cpp(method = "int get_value() const")]
        fn get_value(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "noexcept_basic"]

    class NoexceptMover;

    #[cpp(func = "NoexceptMover* noexcept_mover_new(int)")]
    fn noexcept_mover_new(value: i32) -> NoexceptMover;

    #[cpp(func = "int noexcept_add(int, int)")]
    fn noexcept_add(a: i32, b: i32) -> i32;

    #[cpp(func = "int noexcept_multiply(int, int)")]
    fn noexcept_multiply(a: i32, b: i32) -> i32;

    #[cpp(func = "int throwing_divide(int, int)")]
    fn throwing_divide(a: i32, b: i32) -> i32;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "int check_noexcept(int (*)(int, int))")]
    unsafe fn check_noexcept(fn_: unsafe extern "C" fn(i32, i32) -> i32) -> i32;

    #[cpp(func = "int conditional_abs(int)")]
    fn conditional_abs(value: i32) -> i32;

    #[cpp(func = "NoexceptMover* noexcept_mover_move(NoexceptMover* other)")]
    unsafe fn noexcept_mover_move(other: *mut NoexceptMover) -> *mut NoexceptMover;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "int is_noexcept(int (*)(int, int))")]
    unsafe fn is_noexcept(fn_: unsafe extern "C" fn(i32, i32) -> i32) -> i32;
}

fn main() {
    println!("=== 047_noexcept_basic - noexcept ===\n");

    // noexcept functions
    println!("--- noexcept Functions ---");
    println!("noexcept_add(10, 20) = {}", noexcept_add(10, 20));
    println!("noexcept_multiply(6, 7) = {}", noexcept_multiply(6, 7));
    println!("conditional_abs(-42) = {}", conditional_abs(-42));

    // noexcept move semantics
    println!("\n--- noexcept Move Semantics ---");
    let mut mover1 = noexcept_mover_new(100);
    println!("Original mover created, value = {}", mover1.get_value());
    use hicc::AbiClass;
    let mover2 = unsafe { noexcept_mover_move(&mover1.as_mut_ptr()) };
    println!("Mover moved (noexcept), new value = {}", mover2.get_value());

    println!("\n--- Summary ---");
    println!("1. noexcept declares function won't throw");
    println!("2. Move constructors and move assignment operators often use noexcept");
    println!("3. noexcept move operations have better performance in STL containers");
    println!("4. noexcept functions cannot call potentially throwing functions");
    println!("5. In FFI, noexcept is part of function signature");
}

