hicc::cpp! {
    #include <iostream>
    #include <functional>
    #include <vector>
    #include <thread>
    #include <chrono>

    #include "std_function.h"

    static int hicc_double_fn(int v) { return v * 2; }
    static int hicc_triple_fn(int v) { return v * 3; }

    extern "C" {
        CallbackWrapper* hicc_callback_wrapper_new(int (*fn)(int)) { return new CallbackWrapper(fn); }
        CallbackWrapper* hicc_callback_wrapper_new_double() { return new CallbackWrapper(hicc_double_fn); }
        Processor* hicc_processor_new() { return new Processor(); }
        MultiCallback* hicc_multi_callback_new() { return new MultiCallback(); }
        AsyncProcessor* hicc_async_processor_new() { return new AsyncProcessor(); }
        void hicc_processor_set_double(Processor* p) { p->impl->set_callback(hicc_double_fn); }
        void hicc_multi_callback_add_double(MultiCallback* mc) { mc->impl->add(hicc_double_fn); }
        void hicc_multi_callback_add_triple(MultiCallback* mc) { mc->impl->add(hicc_triple_fn); }
    }
}

hicc::import_class! {
    #[cpp(class = "CallbackWrapper")]
    pub class CallbackWrapper {
        #[cpp(method = "int invoke(int value)")]
        pub fn invoke(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Processor")]
    pub class Processor {
        #[cpp(method = "int process(int value)")]
        pub fn process(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "MultiCallback")]
    pub class MultiCallback {
        #[cpp(method = "void invoke_all(int value)")]
        pub fn invoke_all(&mut self, value: i32);
    }
}

hicc::import_class! {
    #[cpp(class = "AsyncProcessor")]
    pub class AsyncProcessor {
        #[cpp(method = "bool is_cancelled() const")]
        pub fn is_cancelled(&self) -> bool;

        #[cpp(method = "void cancel()")]
        pub fn cancel(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "std_function"]

    class CallbackWrapper;
    class Processor;
    class MultiCallback;
    class AsyncProcessor;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "CallbackWrapper* hicc_callback_wrapper_new(int (*)(int))")]
    pub unsafe fn callback_wrapper_new(fn_: unsafe extern "C" fn(i32) -> i32) -> CallbackWrapper;

    #[cpp(func = "CallbackWrapper* hicc_callback_wrapper_new_double()")]
    pub fn callback_wrapper_new_double() -> CallbackWrapper;

    #[cpp(func = "Processor* hicc_processor_new()")]
    pub fn processor_new() -> Processor;

    #[cpp(func = "MultiCallback* hicc_multi_callback_new()")]
    pub fn multi_callback_new() -> MultiCallback;

    #[cpp(func = "AsyncProcessor* hicc_async_processor_new()")]
    pub fn async_processor_new() -> AsyncProcessor;

    #[cpp(func = "void hicc_processor_set_double(Processor*)")]
    pub unsafe fn processor_set_double(p: *mut Processor);

    #[cpp(func = "void hicc_multi_callback_add_double(MultiCallback*)")]
    pub unsafe fn multi_callback_add_double(mc: *mut MultiCallback);

    #[cpp(func = "void hicc_multi_callback_add_triple(MultiCallback*)")]
    pub unsafe fn multi_callback_add_triple(mc: *mut MultiCallback);
}
