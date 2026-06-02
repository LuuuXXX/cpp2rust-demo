hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <functional>
    #include <vector>
    #include <thread>
    #include <chrono>

    #include "std_function.h"
}

use hicc::AbiClass;

hicc::import_class! {
    #[cpp(class = "CallbackWrapper", destroy = "callback_wrapper_delete")]
    pub class CallbackWrapper {
        #[cpp(method = "int invoke(int value)")]
        fn invoke(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Processor", destroy = "processor_delete")]
    pub class Processor {
        #[cpp(method = "int process(int value)")]
        fn process(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "MultiCallback", destroy = "multi_callback_delete")]
    pub class MultiCallback {
        #[cpp(method = "void invoke_all(int value)")]
        fn invoke_all(&mut self, value: i32);
    }
}

hicc::import_class! {
    #[cpp(class = "AsyncProcessor", destroy = "async_processor_delete")]
    pub class AsyncProcessor {
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

    #[cpp(func = "CallbackWrapper* callback_wrapper_new_double()")]
    fn callback_wrapper_new_double() -> CallbackWrapper;

    #[cpp(func = "Processor* processor_new()")]
    fn processor_new() -> Processor;

    #[cpp(func = "MultiCallback* multi_callback_new()")]
    fn multi_callback_new() -> MultiCallback;

    #[cpp(func = "AsyncProcessor* async_processor_new()")]
    fn async_processor_new() -> AsyncProcessor;

    #[cpp(func = "void processor_set_double(Processor* p)")]
    unsafe fn processor_set_double(p: *mut Processor);

    #[cpp(func = "void multi_callback_add_double(MultiCallback* mc)")]
    unsafe fn multi_callback_add_double(mc: *mut MultiCallback);

    #[cpp(func = "void multi_callback_add_triple(MultiCallback* mc)")]
    unsafe fn multi_callback_add_triple(mc: *mut MultiCallback);
}

fn main() {
    println!("=== 040_std_function - std::function 回调 ===\n");

    println!("--- CallbackWrapper Demo ---");
    let mut wrapper = callback_wrapper_new_double();
    println!("invoke(5) = {} (doubles input)", wrapper.invoke(5));
    println!("invoke(7) = {} (doubles input)", wrapper.invoke(7));

    println!("\n--- Processor Demo ---");
    let mut processor = processor_new();
    unsafe { processor_set_double(&processor.as_mut_ptr()); }
    println!("process(10) = {}", processor.process(10));

    println!("\n--- MultiCallback Demo ---");
    let mut mc = multi_callback_new();
    unsafe {
        multi_callback_add_double(&mc.as_mut_ptr());
        multi_callback_add_triple(&mc.as_mut_ptr());
    }
    println!("Invoking all callbacks with 4:");
    mc.invoke_all(4);

    println!("\n--- AsyncProcessor Demo ---");
    let mut ap = async_processor_new();
    println!("is_cancelled = {}", ap.is_cancelled());
    ap.cancel();
    println!("after cancel: is_cancelled = {}", ap.is_cancelled());

    println!("\nRust FFI: std::function 回调映射");
    println!("1. std::function 存储可调用对象");
    println!("2. 回调可用于事件处理");
    println!("3. 此示例展示基本的回调封装模式");
}

