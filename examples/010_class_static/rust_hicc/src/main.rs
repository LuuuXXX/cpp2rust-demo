hicc::cpp! {
    #include <iostream>

    class Counter {
        int value;
        static int instance_count;
    public:
        Counter() : value(0) {
    instance_count++;
}
        ~Counter() {
    instance_count--;
}
        int getValue() const {
    return value;
}
        void increment() {
    value++;
}
        static int getInstanceCount() {
    return instance_count;
}
        static void resetInstanceCount() {
    instance_count = 0;
}
    };

    int Counter::instance_count = 0;

    Counter* counter_new() {
        return new Counter();
    }

    void counter_delete(Counter* self) {
        delete self;
    }

    int counter_getInstanceCount() {
        return Counter::getInstanceCount();
    }

    void counter_resetInstanceCount() {
        Counter::resetInstanceCount();
    }
}

hicc::import_lib! {
    #![link_name = "class_static"]

    class Counter;

    #[cpp(class = "Counter")]
    class Counter {
        #[cpp(method = "int getValue() const")]
        fn get_value(&self) -> i32;

        #[cpp(method = "void increment()")]
        fn increment(&mut self);

        #[cpp(func = "Counter* counter_new()")]
        fn new() -> *mut Counter;

        #[cpp(func = "void counter_delete(Counter* self)")]
        unsafe fn delete(self_: *mut Counter);

        #[cpp(func = "int counter_getInstanceCount()")]
        fn get_instance_count() -> i32;

        #[cpp(func = "void counter_resetInstanceCount()")]
        fn reset_instance_count();
    }
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

    unsafe {
        counter_delete(&c1);
    }
    println!("Instance count after deleting c1: {}", counter_get_instance_count());

    unsafe {
        counter_delete(&c2);
        counter_delete(&c3);
    }
    println!("Instance count after deleting all: {}", counter_get_instance_count());

    counter_reset_instance_count();
    println!("Instance count after reset: {}", counter_get_instance_count());

    println!("\nRust FFI: Static members work!");
}
