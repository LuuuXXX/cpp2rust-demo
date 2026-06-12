hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <functional>
    #include <vector>
    #include <thread>
    #include <chrono>

    #include "std_function.h"
}

hicc::import_class! {
    #[cpp(class = "CallbackWrapper", destroy = "callback_wrapper_delete")]
    pub class CallbackWrapper {
        #[cpp(method = "int invoke(int value)")]
        pub fn invoke(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Processor", destroy = "processor_delete")]
    pub class Processor {
        #[cpp(method = "int process(int value)")]
        pub fn process(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "MultiCallback", destroy = "multi_callback_delete")]
    pub class MultiCallback {
        #[cpp(method = "void invoke_all(int value)")]
        pub fn invoke_all(&mut self, value: i32);
    }
}

hicc::import_class! {
    #[cpp(class = "AsyncProcessor", destroy = "async_processor_delete")]
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
    #[cpp(func = "CallbackWrapper* callback_wrapper_new(int (*)(int))")]
    pub unsafe fn callback_wrapper_new(fn_: unsafe extern "C" fn(i32) -> i32) -> CallbackWrapper;

    #[cpp(func = "CallbackWrapper* callback_wrapper_new_double()")]
    pub fn callback_wrapper_new_double() -> CallbackWrapper;

    #[cpp(func = "Processor* processor_new()")]
    pub fn processor_new() -> Processor;

    #[cpp(func = "MultiCallback* multi_callback_new()")]
    pub fn multi_callback_new() -> MultiCallback;

    #[cpp(func = "AsyncProcessor* async_processor_new()")]
    pub fn async_processor_new() -> AsyncProcessor;

    #[cpp(func = "void processor_set_double(Processor* p)")]
    pub unsafe fn processor_set_double(p: *mut Processor);

    #[cpp(func = "void multi_callback_add_double(MultiCallback* mc)")]
    pub unsafe fn multi_callback_add_double(mc: *mut MultiCallback);

    #[cpp(func = "void multi_callback_add_triple(MultiCallback* mc)")]
    pub unsafe fn multi_callback_add_triple(mc: *mut MultiCallback);
}
