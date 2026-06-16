//! 047_noexcept_basic: noexcept 基本函数与移动类型（命名空间类直接持有状态）。
//!
//! `NoexceptMover` 演示 move-only 类型的 `noexcept` 移动语义；自由函数演示 `noexcept`
//! 与内部捕获异常的安全包装。hicc 直出无需 extern-C 不透明指针 + `*_delete`，析构由 Rust `Drop` 自动完成。

hicc::cpp! {
    #include "noexcept_basic.h"
}

hicc::import_class! {
    #[cpp(class = "noexcept_basic_ns::NoexceptMover")]
    pub class NoexceptMover {
        #[cpp(method = "int get_value() const")]
        pub fn get_value(&self) -> i32;

        pub fn new(v: i32) -> Self { noexcept_mover_new(v) }
    }
}

hicc::import_lib! {
    #![link_name = "noexcept_basic"]

    #[cpp(func = "std::unique_ptr<noexcept_basic_ns::NoexceptMover> hicc::make_unique<noexcept_basic_ns::NoexceptMover, int>(int&&)")]
    pub fn noexcept_mover_new(v: i32) -> NoexceptMover;

    #[cpp(func = "int noexcept_basic_ns::noexcept_add(int, int)")]
    pub fn noexcept_add(a: i32, b: i32) -> i32;

    #[cpp(func = "int noexcept_basic_ns::noexcept_multiply(int, int)")]
    pub fn noexcept_multiply(a: i32, b: i32) -> i32;

    #[cpp(func = "int noexcept_basic_ns::conditional_abs(int)")]
    pub fn conditional_abs(x: i32) -> i32;

    #[cpp(func = "int noexcept_basic_ns::safe_divide(int, int)")]
    pub fn safe_divide(a: i32, b: i32) -> i32;
}
