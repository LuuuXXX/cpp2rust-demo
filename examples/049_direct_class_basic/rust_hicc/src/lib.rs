hicc::cpp! {
    #include "direct_class_basic.h"
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
    #![link_name = "direct_class_basic"]

    class Counter;

    #[cpp(func = "std::unique_ptr<Counter> hicc::make_unique<Counter>()")]
    pub fn counter_new() -> Counter;
}
