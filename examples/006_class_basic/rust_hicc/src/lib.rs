hicc::cpp! {
    #include <iostream>

    #include "class_basic.h"
}

hicc::import_class! {
    #[cpp(class = "Counter", destroy = "counter_delete")]
    pub class Counter {
        #[cpp(method = "int get() const")]
        pub fn get(&self) -> i32;

        #[cpp(method = "void increment()")]
        pub fn increment(&mut self);

        #[cpp(method = "void decrement()")]
        pub fn decrement(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "class_basic"]

    class Counter;

    #[cpp(func = "Counter* counter_new()")]
    pub fn counter_new() -> Counter;
}
