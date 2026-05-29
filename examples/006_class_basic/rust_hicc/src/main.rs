hicc::cpp! {
    #include <iostream>

    #include "class_basic.h"
}

hicc::import_class! {
    #[cpp(class = "Counter", destroy = "counter_delete")]
    class Counter {
        #[cpp(method = "int get() const")]
        fn get(&self) -> i32;

        #[cpp(method = "void increment()")]
        fn increment(&mut self);

        #[cpp(method = "void decrement()")]
        fn decrement(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "class_basic"]

    class Counter;

    #[cpp(func = "Counter* counter_new()")]
    fn counter_new() -> Counter;
}

fn main() {
    let mut counter = counter_new();
    println!("Initial value: {}", counter.get());

    counter.increment();
    counter.increment();
    counter.increment();
    println!("After 3 increments: {}", counter.get());

    counter.decrement();
    println!("After 1 decrement: {}", counter.get());

    println!("\nRust FFI: Basic class operations completed!");
}

