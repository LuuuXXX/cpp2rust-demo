use tuple_basic::*;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    use std::ffi::CString;

    println!("=== 038_tuple_basic - std::tuple ===\n");

    // Tuple2 demo
    println!("--- Tuple2 (int, string) Demo ---");
    let second = CString::new("hello").unwrap();
    let tuple = unsafe { tuple2_new(42, second.as_ptr()) };
    println!("Tuple2: first={}, second={}", tuple.get_first(), decode_cstr(tuple.get_second()));

    println!();

    // Tuple3 demo
    println!("--- Tuple3 (int, double, string) Demo ---");
    let third = CString::new("world").unwrap();
    let tuple = unsafe { tuple3_new(100, 3.14159, third.as_ptr()) };
    println!("Tuple3: first={}, second={}, third={}", tuple.get_first(), tuple.get_second(), decode_cstr(tuple.get_third()));

    println!();

    // Tuple4 demo
    println!("--- Tuple4 (int, double, string, int) Demo ---");
    let third = CString::new("tuple").unwrap();
    let tuple = unsafe { tuple4_new(1, 2.71828, third.as_ptr(), 4) };
    println!("Tuple4 elements:");
    println!("  [0] = {}", tuple.get_first());
    println!("  [1] = {}", tuple.get_second());
    println!("  [2] = {}", decode_cstr(tuple.get_third()));
    println!("  [3] = {}", tuple.get_fourth());

    println!("\nRust FFI: std::tuple 映射");
    println!("1. std::tuple 是异构容器的编译时固定版本");
    println!("2. 通过 std::get<N>(tuple) 访问元素");
    println!("3. FFI 需要为每个元素类型提供独立的 getter 函数");
    println!("4. 字符串等复杂类型需要额外的内存管理");
}
