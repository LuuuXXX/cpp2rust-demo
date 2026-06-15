hicc::cpp! {
    #include <iostream>
    #include <functional>
    #include <vector>
    #include <thread>
    #include <chrono>

    #include "std_function.h"
}

hicc::import_class! {
    #[cpp(class = "CallbackWrapperImpl")]
    pub class CallbackWrapperImpl {
        #[cpp(method = "int invoke(int value)")]
        pub fn invoke(&mut self, value: i32) -> i32;

        // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
        #[cpp(method = "void set(int (*)(int) fn)")]
        pub fn set(&mut self, fn_: unsafe extern "C" fn(i32) -> i32);
    }
}

hicc::import_class! {
    #[cpp(class = "ProcessorImpl")]
    pub class ProcessorImpl {
        // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
        #[cpp(method = "void set_callback(int (*)(int) cb)")]
        pub fn set_callback(&mut self, cb: unsafe extern "C" fn(i32) -> i32);

        #[cpp(method = "int process(int value)")]
        pub fn process(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "MultiCallbackImpl")]
    pub class MultiCallbackImpl {
        // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
        #[cpp(method = "void add(int (*)(int) cb)")]
        pub fn add(&mut self, cb: unsafe extern "C" fn(i32) -> i32);

        #[cpp(method = "void invoke_all(int value)")]
        pub fn invoke_all(&mut self, value: i32);
    }
}

hicc::import_class! {
    #[cpp(class = "AsyncProcessorImpl")]
    pub class AsyncProcessorImpl {
        // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
        #[cpp(method = "void set_completion_callback(void (*)(int, int) cb)")]
        pub fn set_completion_callback(&mut self, cb: unsafe extern "C" fn(i32, i32));

        // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
        #[cpp(method = "void set_progress_callback(void (*)(int) cb)")]
        pub fn set_progress_callback(&mut self, cb: unsafe extern "C" fn(i32));

        #[cpp(method = "void start(int value)")]
        pub fn start(&mut self, value: i32);

        #[cpp(method = "void cancel()")]
        pub fn cancel(&mut self);
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

    class CallbackWrapperImpl;
    class ProcessorImpl;
    class MultiCallbackImpl;
    class AsyncProcessorImpl;
    class CallbackWrapper;
    class Processor;
    class MultiCallback;
    class AsyncProcessor;

    #[cpp(func = "std::unique_ptr<CallbackWrapperImpl> std::make_unique<CallbackWrapperImpl>(int (*)(int))")]
    pub unsafe fn callback_wrapper_impl_new_with_fn_(fn_: unsafe extern "C" fn(i32) -> i32) -> CallbackWrapperImpl;

    #[cpp(func = "std::unique_ptr<ProcessorImpl> hicc::make_unique<ProcessorImpl>()")]
    pub fn processor_impl_new() -> ProcessorImpl;

    #[cpp(func = "std::unique_ptr<MultiCallbackImpl> hicc::make_unique<MultiCallbackImpl>()")]
    pub fn multi_callback_impl_new() -> MultiCallbackImpl;

    #[cpp(func = "std::unique_ptr<AsyncProcessorImpl> hicc::make_unique<AsyncProcessorImpl>()")]
    pub fn async_processor_impl_new() -> AsyncProcessorImpl;

    #[cpp(func = "std::unique_ptr<CallbackWrapper> std::make_unique<CallbackWrapper>(int (*)(int))")]
    pub unsafe fn callback_wrapper_new_with_fn_(fn_: unsafe extern "C" fn(i32) -> i32) -> CallbackWrapper;

    #[cpp(func = "std::unique_ptr<Processor> hicc::make_unique<Processor>()")]
    pub fn processor_new() -> Processor;

    #[cpp(func = "std::unique_ptr<MultiCallback> hicc::make_unique<MultiCallback>()")]
    pub fn multi_callback_new() -> MultiCallback;

    #[cpp(func = "std::unique_ptr<AsyncProcessor> hicc::make_unique<AsyncProcessor>()")]
    pub fn async_processor_new() -> AsyncProcessor;
}
