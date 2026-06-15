use std_function::*;
use hicc::AbiClass;

fn main() {
    println!("=== 040_std_function - std::function 回调 ===\n");

    println!("--- CallbackWrapper Demo ---");
    let mut wrapper = callback_wrapper_new_double();
    println!("invoke(5) = {} (doubles input)", wrapper.invoke(5));
    println!("invoke(7) = {} (doubles input)", wrapper.invoke(7));

    println!("\n--- Processor Demo ---");
    let mut processor = processor_new();
    unsafe { processor_set_double(processor.as_mut_ptr()); }
    println!("process(10) = {}", processor.process(10));

    println!("\n--- MultiCallback Demo ---");
    let mut mc = multi_callback_new();
    unsafe {
        multi_callback_add_double(mc.as_mut_ptr());
        multi_callback_add_triple(mc.as_mut_ptr());
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
