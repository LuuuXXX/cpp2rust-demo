hicc::cpp! {
    #include <iostream>

    #include "class_static.h"
}

hicc::import_class! {
    #[cpp(class = "Counter")]
    pub class Counter {
        #[cpp(method = "int getValue() const")]
        pub fn get_value(&self) -> i32;

        #[cpp(method = "void increment()")]
        pub fn increment(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "class_static"]

    class Counter;

    #[cpp(func = "int Counter::getInstanceCount()")]
    pub fn counter_get_instance_count() -> i32;

    #[cpp(func = "void Counter::resetInstanceCount()")]
    pub fn counter_reset_instance_count();

    #[cpp(func = "std::unique_ptr<Counter> hicc::make_unique<Counter>()")]
    pub fn counter_new() -> Counter;
}
