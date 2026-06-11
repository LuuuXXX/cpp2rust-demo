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
        pub fn get_value(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "noexcept_basic"]

    class NoexceptMover;

    #[cpp(func = "NoexceptMover* noexcept_mover_new(int)")]
    pub fn noexcept_mover_new(value: i32) -> NoexceptMover;

    #[cpp(func = "int noexcept_add(int, int)")]
    pub fn noexcept_add(a: i32, b: i32) -> i32;

    #[cpp(func = "int noexcept_multiply(int, int)")]
    pub fn noexcept_multiply(a: i32, b: i32) -> i32;

    #[cpp(func = "int throwing_divide(int, int)")]
    pub fn throwing_divide(a: i32, b: i32) -> i32;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "int check_noexcept(int (*)(int, int))")]
    pub unsafe fn check_noexcept(fn_: unsafe extern "C" fn(i32, i32) -> i32) -> i32;

    #[cpp(func = "int conditional_abs(int)")]
    pub fn conditional_abs(value: i32) -> i32;

    #[cpp(func = "NoexceptMover* noexcept_mover_move(NoexceptMover* other)")]
    pub unsafe fn noexcept_mover_move(other: *mut NoexceptMover) -> *mut NoexceptMover;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "int is_noexcept(int (*)(int, int))")]
    pub unsafe fn is_noexcept(fn_: unsafe extern "C" fn(i32, i32) -> i32) -> i32;
}
