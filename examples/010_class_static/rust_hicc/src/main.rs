hicc::cpp! {
    #include <iostream>

    #include "class_static.h"
}

hicc::import_class! {
    #[cpp(class = "Counter", destroy = "counter_delete")]
    class Counter {
        #[cpp(method = "int getValue() const")]
        fn get_value(&self) -> i32;

        #[cpp(method = "void increment()")]
        fn increment(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "class_static"]

    class Counter;

    #[cpp(func = "Counter* counter_new()")]
    fn counter_new() -> Counter;

    #[cpp(func = "int counter_getInstanceCount()")]
    fn counter_get_instance_count() -> i32;

    #[cpp(func = "void counter_resetInstanceCount()")]
    fn counter_reset_instance_count();
}

fn main() {
    println!("Initial instance count: {}", counter_get_instance_count());

    let mut c1 = counter_new();
    let mut c2 = counter_new();
    let mut c3 = counter_new();

    println!("Instance count after creating 3: {}", counter_get_instance_count());

    c1.increment();
    c1.increment();
    c2.increment();

    println!("c1 value: {}", c1.get_value());
    println!("c2 value: {}", c2.get_value());
    println!("c3 value: {}", c3.get_value());

    println!("Instance count after deleting c1: {}", counter_get_instance_count());

    println!("Instance count after deleting all: {}", counter_get_instance_count());

    counter_reset_instance_count();
    println!("Instance count after reset: {}", counter_get_instance_count());

    println!("\nRust FFI: Static members work!");
}

