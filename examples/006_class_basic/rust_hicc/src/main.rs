hicc::cpp! {
    #include <iostream>

    class Counter {
        int value = 0;
    public:
        Counter() : value(0) {}
        ~Counter() {}
        int get() const { return value; }
        void increment() { value++; }
        void decrement() { value--; }
    };

    Counter* counter_new() { return new Counter(); }

    void counter_delete(Counter* self) { delete self; }
}

hicc::import_class! {
    #[cpp(class = "Counter")]
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
    fn counter_new() -> *mut Counter;

    #[cpp(func = "void counter_delete(Counter* self)")]
    unsafe fn counter_delete(self_: *mut Counter);
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

    unsafe {
        counter_delete(&counter);
    }
    println!("\nRust FFI: Basic class operations completed!");
}

