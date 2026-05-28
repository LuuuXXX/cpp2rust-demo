hicc::cpp! {
    #include <stddef.h>
    #include <iostream>

    struct CallbackWrapper {
    public:
        int multiplier;
        CallbackWrapper(int m) : multiplier(m) {}
        ~CallbackWrapper() {}
        int process(int value) const { return value * multiplier; }
        int get_value() const { return multiplier; }
    };

    struct Processor {
    public:
        int input;
        Processor() : input(0) {}
        ~Processor() {}
        void set_input(int v) { input = v; }
        int get_input() const { return input; }
    };

    struct MultiCallback {
    public:
        MultiCallback() {}
        ~MultiCallback() {}
        int size() const { return 0; }
    };

    struct AsyncProcessor {
    public:
        AsyncProcessor() {}
        ~AsyncProcessor() {}
        int size() const { return 0; }
    };

    CallbackWrapper* callback_wrapper_new(int multiplier) {
        return new CallbackWrapper(multiplier);
    }

    void callback_wrapper_delete(CallbackWrapper* self) {
        delete self;
    }

    Processor* processor_new() {
        return new Processor();
    }

    void processor_delete(Processor* self) {
        delete self;
    }

    MultiCallback* multi_callback_new() {
        return new MultiCallback();
    }

    void multi_callback_delete(MultiCallback* self) {
        delete self;
    }

    AsyncProcessor* async_processor_new() {
        return new AsyncProcessor();
    }

    void async_processor_delete(AsyncProcessor* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "CallbackWrapper")]
    class CallbackWrapper {
        #[cpp(method = "int process(int value) const")]
        fn process(&self, value: i32) -> i32;

        #[cpp(method = "int get_value() const")]
        fn get_value(&self) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Processor")]
    class Processor {
        #[cpp(method = "void set_input(int v)")]
        fn set_input(&mut self, v: i32);

        #[cpp(method = "int get_input() const")]
        fn get_input(&self) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "MultiCallback")]
    class MultiCallback {
        #[cpp(method = "int size() const")]
        fn size(&self) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "AsyncProcessor")]
    class AsyncProcessor {
        #[cpp(method = "int size() const")]
        fn size(&self) -> i32;
    }
}


hicc::import_lib! {
    #![link_name = "std_function"]

    class CallbackWrapper;
    class Processor;
    class MultiCallback;
    class AsyncProcessor;

    #[cpp(func = "CallbackWrapper* callback_wrapper_new(int)")]
    fn callback_wrapper_new(multiplier: i32) -> *mut CallbackWrapper;

    #[cpp(func = "void callback_wrapper_delete(CallbackWrapper* self)")]
    unsafe fn callback_wrapper_delete(self_: *mut CallbackWrapper);

    #[cpp(func = "Processor* processor_new()")]
    fn processor_new() -> *mut Processor;

    #[cpp(func = "void processor_delete(Processor* self)")]
    unsafe fn processor_delete(self_: *mut Processor);

    #[cpp(func = "MultiCallback* multi_callback_new()")]
    fn multi_callback_new() -> *mut MultiCallback;

    #[cpp(func = "void multi_callback_delete(MultiCallback* self)")]
    unsafe fn multi_callback_delete(self_: *mut MultiCallback);

    #[cpp(func = "AsyncProcessor* async_processor_new()")]
    fn async_processor_new() -> *mut AsyncProcessor;

    #[cpp(func = "void async_processor_delete(AsyncProcessor* self)")]
    unsafe fn async_processor_delete(self_: *mut AsyncProcessor);
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



