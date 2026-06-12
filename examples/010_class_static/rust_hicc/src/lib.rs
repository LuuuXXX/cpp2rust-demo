hicc::cpp! {
    #include <iostream>

    #include "class_static.h"
}

hicc::import_class! {
    #[cpp(class = "Counter", destroy = "counter_delete")]
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

    #[cpp(func = "Counter* counter_new()")]
    pub fn counter_new() -> Counter;

    #[cpp(func = "int counter_getInstanceCount()")]
    pub fn counter_get_instance_count() -> i32;

    #[cpp(func = "void counter_resetInstanceCount()")]
    pub fn counter_reset_instance_count();
}
