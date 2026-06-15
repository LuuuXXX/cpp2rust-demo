hicc::cpp! {
    #include <cstddef>
    #include <iostream>
    #include <stdexcept>
    #include <utility>

    #include "noexcept_basic.h"

    std::unique_ptr<NoexceptMover> _cpp2rust_make_unique_noexcept_mover_with_value(int value) { return std::make_unique<NoexceptMover>(value); }
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
    pub fn noexcept_mover_new_with_value(value: i32) -> NoexceptMover;
}
