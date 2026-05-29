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
        int invoke(int value) { return impl->invoke(value); }
    };

    struct Processor {
    public:
        ProcessorImpl* impl;
        Processor() : impl(new ProcessorImpl()) {}
        ~Processor() { delete impl; }
        int process(int value) { return impl->process(value); }
    };

    struct MultiCallback {
    public:
        MultiCallbackImpl* impl;
        MultiCallback() : impl(new MultiCallbackImpl()) {}
        ~MultiCallback() { delete impl; }
        void invoke_all(int value) { impl->invoke_all(value); }
    };

    struct AsyncProcessor {
    public:
        AsyncProcessorImpl* impl;
        AsyncProcessor() : impl(new AsyncProcessorImpl()) {}
        ~AsyncProcessor() { delete impl; }
        bool is_cancelled() const { return impl->cancelled; }
        void cancel() { impl->cancel(); }
    };

    CallbackWrapper* callback_wrapper_new(int (*fn)(int)) {
        return new CallbackWrapper(fn);
    }

    CallbackWrapper* callback_wrapper_new_double() {
        return new CallbackWrapper([](int x) -> int { return x * 2; });
    }

    void callback_wrapper_delete(CallbackWrapper* self) {
        delete self;
    }

    Processor* processor_new() {
        return new Processor();
    }

    void processor_set_double(Processor* p) {
        p->impl->set_callback([](int x) -> int { return x * 2; });
    }

    void processor_delete(Processor* self) {
        delete self;
    }

    MultiCallback* multi_callback_new() {
        return new MultiCallback();
    }

    void multi_callback_add_double(MultiCallback* mc) {
        mc->impl->add([](int x) -> int { return x * 2; });
    }

    void multi_callback_add_triple(MultiCallback* mc) {
        mc->impl->add([](int x) -> int { return x * 3; });
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
        #[cpp(method = "int invoke(int value)")]
        fn invoke(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Processor")]
    class Processor {
        #[cpp(method = "int process(int value)")]
        fn process(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "MultiCallback")]
    class MultiCallback {
        #[cpp(method = "void invoke_all(int value)")]
        fn invoke_all(&mut self, value: i32);
    }
}

hicc::import_class! {
    #[cpp(class = "AsyncProcessor")]
    class AsyncProcessor {
        #[cpp(method = "bool is_cancelled() const")]
        fn is_cancelled(&self) -> bool;

        #[cpp(method = "void cancel()")]
        fn cancel(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "std_function"]

    class CallbackWrapper;
    class Processor;
    class MultiCallback;
    class AsyncProcessor;

    #[cpp(func = "Processor* processor_new()")]
    fn processor_new() -> *mut Processor;

    #[cpp(func = "void processor_delete(Processor* self)")]
    unsafe fn processor_delete(self_: *mut Processor);

    #[cpp(func = "CallbackWrapper* callback_wrapper_new_double()")]
    fn callback_wrapper_new_double() -> *mut CallbackWrapper;

    #[cpp(func = "void callback_wrapper_delete(CallbackWrapper* self)")]
    unsafe fn callback_wrapper_delete(self_: *mut CallbackWrapper);

    #[cpp(func = "void processor_set_double(Processor* p)")]
    unsafe fn processor_set_double(p: *mut Processor);

    #[cpp(func = "MultiCallback* multi_callback_new()")]
    fn multi_callback_new() -> *mut MultiCallback;

    #[cpp(func = "void multi_callback_add_double(MultiCallback* mc)")]
    unsafe fn multi_callback_add_double(mc: *mut MultiCallback);

    #[cpp(func = "void multi_callback_add_triple(MultiCallback* mc)")]
    unsafe fn multi_callback_add_triple(mc: *mut MultiCallback);

    #[cpp(func = "void multi_callback_delete(MultiCallback* self)")]
    unsafe fn multi_callback_delete(self_: *mut MultiCallback);

    #[cpp(func = "AsyncProcessor* async_processor_new()")]
    fn async_processor_new() -> *mut AsyncProcessor;

    #[cpp(func = "void async_processor_delete(AsyncProcessor* self)")]
    unsafe fn async_processor_delete(self_: *mut AsyncProcessor);
}

fn main() {
    println!("=== 040_std_function - std::function 回调 ===\n");

    println!("--- CallbackWrapper Demo ---");
    let mut wrapper = callback_wrapper_new_double();
    println!("invoke(5) = {} (doubles input)", wrapper.invoke(5));
    println!("invoke(7) = {} (doubles input)", wrapper.invoke(7));
    unsafe { callback_wrapper_delete(&wrapper); }

    println!("\n--- Processor Demo ---");
    let mut processor = processor_new();
    unsafe { processor_set_double(&processor); }
    println!("process(10) = {}", processor.process(10));
    unsafe { processor_delete(&processor); }

    println!("\n--- MultiCallback Demo ---");
    let mut mc = multi_callback_new();
    unsafe {
        multi_callback_add_double(&mc);
        multi_callback_add_triple(&mc);
    }
    println!("Invoking all callbacks with 4:");
    mc.invoke_all(4);
    unsafe { multi_callback_delete(&mc); }

    println!("\n--- AsyncProcessor Demo ---");
    let mut ap = async_processor_new();
    println!("is_cancelled = {}", ap.is_cancelled());
    ap.cancel();
    println!("after cancel: is_cancelled = {}", ap.is_cancelled());
    unsafe { async_processor_delete(&ap); }

    println!("\nRust FFI: std::function 回调映射");
    println!("1. std::function 存储可调用对象");
    println!("2. 回调可用于事件处理");
    println!("3. 此示例展示基本的回调封装模式");
}


