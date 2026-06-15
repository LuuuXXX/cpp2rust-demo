hicc::cpp! {
    #include <cstddef>
    #include <iostream>
    #include <stdexcept>
    #include <utility>
    #include <memory>

    #include "noexcept_basic.h"

    std::unique_ptr<NoexceptMover> _cpp2rust_make_unique_noexcept_mover_with_value(int value) { return std::make_unique<NoexceptMover>(value); }

    std::unique_ptr<NoexceptMover> noexcept_mover_move(NoexceptMover* other) {
        return std::make_unique<NoexceptMover>(std::move(*other));
    }

    int noexcept_add(int a, int b) {
        return a + b;
    }

    int noexcept_multiply(int a, int b) {
        return a * b;
    }

    int conditional_abs(int value) {
        return value >= 0 ? value : -value;
    }
}

hicc::import_class! {
    #[cpp(class = "NoexceptMover")]
    pub class NoexceptMover {
        #[cpp(method = "int get_value() const")]
        pub fn get_value(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "noexcept_basic"]

    class NoexceptMover;

    #[cpp(func = "std::unique_ptr<NoexceptMover> _cpp2rust_make_unique_noexcept_mover_with_value(int)")]
    pub fn noexcept_mover_new(value: i32) -> NoexceptMover;

    #[cpp(func = "std::unique_ptr<NoexceptMover> noexcept_mover_move(NoexceptMover*)")]
    pub unsafe fn noexcept_mover_move(other: *mut NoexceptMover) -> NoexceptMover;

    #[cpp(func = "int noexcept_add(int, int)")]
    pub fn noexcept_add(a: i32, b: i32) -> i32;

    #[cpp(func = "int noexcept_multiply(int, int)")]
    pub fn noexcept_multiply(a: i32, b: i32) -> i32;

    #[cpp(func = "int conditional_abs(int)")]
    pub fn conditional_abs(value: i32) -> i32;
}
