hicc::cpp! {
    #include <cstdint>

    #include "summary.h"
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
    #![link_name = "summary"]

    class Counter;

    #[cpp(func = "Counter* counter_new()")]
    pub fn counter_new() -> Counter;

    #[cpp(func = "int safe_add(int, int)")]
    pub fn safe_add(a: i32, b: i32) -> i32;

    #[cpp(func = "int get_max_size()")]
    pub fn get_max_size() -> i32;
}
