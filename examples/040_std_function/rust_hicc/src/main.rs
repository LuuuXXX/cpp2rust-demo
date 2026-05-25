hicc::cpp! {
    #include <iostream>

    // Simple callback wrapper
    class CallbackWrapper {
    public:
        int stored_value;
        int multiplier;
        CallbackWrapper(int mult) : stored_value(0), multiplier(mult) {}
        int process(int input) { stored_value = input * multiplier; return stored_value; }
        int get_value() const { return stored_value; }
    };

    // Processor that stores a callback
    class Processor {
    public:
        int stored_input;
        int stored_result;
        Processor() : stored_input(0), stored_result(0) {}
        void set_input(int input) { stored_input = input; }
        int get_input() const { return stored_input; }
        int get_result() const { return stored_result; }
    };

    CallbackWrapper* callback_wrapper_new(int multiplier) { return new CallbackWrapper(multiplier); }
    void callback_wrapper_delete(CallbackWrapper* self) { delete self; }

    Processor* processor_new() { return new Processor(); }
    void processor_delete(Processor* self) { delete self; }
}

hicc::import_class! {
    #[cpp(class = "CallbackWrapper")]
    class CallbackWrapper {
        #[cpp(method = "int process(int)")]
        fn process(&mut self, input: i32) -> i32;

        #[cpp(method = "int get_value() const")]
        fn get_value(&self) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Processor")]
    class Processor {
        #[cpp(method = "void set_input(int)")]
        fn set_input(&mut self, input: i32);

        #[cpp(method = "int get_input() const")]
        fn get_input(&self) -> i32;

        #[cpp(method = "int get_result() const")]
        fn get_result(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "std_function"]

    class CallbackWrapper;
    class Processor;

    #[cpp(func = "CallbackWrapper* callback_wrapper_new(int)")]
    fn callback_wrapper_new(multiplier: i32) -> *mut CallbackWrapper;

    #[cpp(func = "void callback_wrapper_delete(CallbackWrapper* self)")]
    unsafe fn callback_wrapper_delete(self_: *mut CallbackWrapper);

    #[cpp(func = "Processor* processor_new()")]
    fn processor_new() -> *mut Processor;

    #[cpp(func = "void processor_delete(Processor* self)")]
    unsafe fn processor_delete(self_: *mut Processor);
}

fn main() {
    println!("=== 040_std_function - std::function 回调 ===\n");

    // CallbackWrapper example
    println!("--- CallbackWrapper Demo ---");
    unsafe {
        let mut wrapper = callback_wrapper_new(2);
        let result = wrapper.process(5);
        println!("process(5) with multiplier=2: {}", result);
        println!("get_value(): {}", wrapper.get_value());
        callback_wrapper_delete(&wrapper);

        let mut wrapper = callback_wrapper_new(3);
        let result = wrapper.process(7);
        println!("process(7) with multiplier=3: {}", result);
        callback_wrapper_delete(&wrapper);
    }

    // Processor example
    println!("\n--- Processor Demo ---");
    unsafe {
        let mut processor = processor_new();

        processor.set_input(21);
        println!("Set input: {}", processor.get_input());

        // Simulate processing
        let result = processor.get_input() * 2;
        println!("Simulated result (input * 2): {}", result);

        processor_delete(&processor);
    }

    println!("\nRust FFI: std::function 回调映射");
    println!("1. std::function 存储可调用对象");
    println!("2. 回调可用于事件处理");
    println!("3. 此示例展示基本的回调封装模式");
}
