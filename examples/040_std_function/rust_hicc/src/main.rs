hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <functional>
    #include <vector>
    #include <thread>
    #include <chrono>

    class CallbackWrapperImpl {
    public:
        std::function<int(int)> callback;
    public:
        CallbackWrapperImpl(int (*fn)(int)) : callback(fn) {}
        ~CallbackWrapperImpl() {}
        int invoke(int value) {
    if (callback) return callback(value);
    return value;
}
        void set(int (*fn)(int)) {
    callback = fn;
}
    };

    class ProcessorImpl {
    public:
        std::function<int(int)> callback;
    public:
        ProcessorImpl() : callback(nullptr) {}
        ~ProcessorImpl() {}
        void set_callback(int (*cb)(int)) {
    callback = cb;
}
        int process(int value) {
    if (callback) return callback(value);
    return value;
}
    };

    class MultiCallbackImpl {
    public:
        std::vector<std::function<int(int)>> callbacks;
    public:
        MultiCallbackImpl() {}
        ~MultiCallbackImpl() {}
        void add(int (*cb)(int)) {
    callbacks.push_back(cb);
}
        void invoke_all(int value) {
    for (auto& cb : callbacks) {
        cb(value);
    }
}
    };

    class AsyncProcessorImpl {
    public:
        std::function<void(int, int)> completion_callback;
        std::function<void(int)> progress_callback;
        bool cancelled;
    public:
        AsyncProcessorImpl() : cancelled(false) {}
        ~AsyncProcessorImpl() {}
        void set_completion_callback(void (*cb)(int, int)) {
    completion_callback = cb;
}
        void set_progress_callback(void (*cb)(int)) {
    progress_callback = cb;
}
        void start(int value) {
    cancelled = false;
    // Simulate async processing
    for (int i = 0; i <= 100; i += 20) {
        if (cancelled) break;
        if (progress_callback) progress_callback(i);
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
    }
    if (completion_callback) completion_callback(value, value * 2);
}
        void cancel() {
    cancelled = true;
}
    };

    struct CallbackWrapper {
    public:
        CallbackWrapperImpl* impl;
        CallbackWrapper(int (*fn)(int)) : impl(new CallbackWrapperImpl(fn)) {}
        ~CallbackWrapper() { delete impl; }
    };

    struct Processor {
    public:
        ProcessorImpl* impl;
        Processor() : impl(new ProcessorImpl()) {}
        ~Processor() { delete impl; }
    };

    struct MultiCallback {
    public:
        MultiCallbackImpl* impl;
        MultiCallback() : impl(new MultiCallbackImpl()) {}
        ~MultiCallback() { delete impl; }
    };

    struct AsyncProcessor {
    public:
        AsyncProcessorImpl* impl;
        AsyncProcessor() : impl(new AsyncProcessorImpl()) {}
        ~AsyncProcessor() { delete impl; }
    };

    CallbackWrapper* callback_wrapper_new(int (*fn)(int)) {
        return new CallbackWrapper(fn);
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

hicc::import_lib! {
    #![link_name = "std_function"]

    class CallbackWrapper;
    class Processor;
    class MultiCallback;
    class AsyncProcessor;

    #[cpp(func = "CallbackWrapper* callback_wrapper_new(int (*)(int))")]
    fn callback_wrapper_new(fn_: int (*)(int)) -> *mut CallbackWrapper;

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

