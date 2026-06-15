hicc::cpp! {
    #include <cstdint>

    #include "summary.h"

    int safe_add(int a, int b) {
        return a + b;
    }
    int get_max_size() {
        return 100;
    }
}

hicc::import_class! {
    #[cpp(class = "Counter")]
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

    #[cpp(func = "std::unique_ptr<Counter> hicc::make_unique<Counter>()")]
    pub fn counter_new() -> Counter;

    #[cpp(func = "int safe_add(int, int)")]
    pub fn safe_add(a: i32, b: i32) -> i32;

    #[cpp(func = "int get_max_size()")]
    pub fn get_max_size() -> i32;
}
