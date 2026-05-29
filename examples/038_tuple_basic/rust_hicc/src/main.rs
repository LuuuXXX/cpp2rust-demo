hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <tuple>
    #include <string>
    #include <cstring>

    #include "tuple_basic.h"
}

hicc::import_class! {
    #[cpp(class = "Tuple2", destroy = "tuple2_delete")]
    class Tuple2 {
        #[cpp(method = "int get_first() const")]
        fn get_first(&self) -> i32;

        #[cpp(method = "const char* get_second() const")]
        fn get_second(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Tuple3", destroy = "tuple3_delete")]
    class Tuple3 {
        #[cpp(method = "int get_first() const")]
        fn get_first(&self) -> i32;

        #[cpp(method = "double get_second() const")]
        fn get_second(&self) -> f64;

        #[cpp(method = "const char* get_third() const")]
        fn get_third(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Tuple4", destroy = "tuple4_delete")]
    class Tuple4 {
        #[cpp(method = "int get_first() const")]
        fn get_first(&self) -> i32;

        #[cpp(method = "double get_second() const")]
        fn get_second(&self) -> f64;

        #[cpp(method = "const char* get_third() const")]
        fn get_third(&self) -> *const i8;

        #[cpp(method = "int get_fourth() const")]
        fn get_fourth(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "tuple_basic"]

    class Tuple2;
    class Tuple3;
    class Tuple4;

    #[cpp(func = "Tuple2* tuple2_new(int, const char*)")]
    unsafe fn tuple2_new(first: i32, second: *const i8) -> Tuple2;

    #[cpp(func = "Tuple3* tuple3_new(int, double, const char*)")]
    unsafe fn tuple3_new(first: i32, second: f64, third: *const i8) -> Tuple3;

    #[cpp(func = "Tuple4* tuple4_new(int, double, const char*, int)")]
    unsafe fn tuple4_new(first: i32, second: f64, third: *const i8, fourth: i32) -> Tuple4;

    #[cpp(func = "Tuple2* make_int_string_pair(int, const char*)")]
    unsafe fn make_int_string_pair(i: i32, s: *const i8) -> *mut Tuple2;

    #[cpp(func = "Tuple3* make_int_double_string(int, double, const char*)")]
    unsafe fn make_int_double_string(i: i32, d: f64, s: *const i8) -> *mut Tuple3;
}

fn main() {
    use std::ffi::CString;
    use std::ffi::CStr;

    println!("=== 038_tuple_basic - std::tuple ===\n");

    // Tuple2 demo
    println!("--- Tuple2 (int, string) Demo ---");
    let second = CString::new("hello").unwrap();
    let tuple = unsafe { tuple2_new(42, second.as_ptr()) };

    let first = tuple.get_first();
    let second_ptr = tuple.get_second();
    let second_str = unsafe { CStr::from_ptr(second_ptr).to_string_lossy() };

    println!("Tuple2: first={}, second={}", first, second_str);

    println!();

    // Tuple3 demo
    println!("--- Tuple3 (int, double, string) Demo ---");
    let third = CString::new("world").unwrap();
    let tuple = unsafe { tuple3_new(100, 3.14159, third.as_ptr()) };

    let first = tuple.get_first();
    let second = tuple.get_second();
    let third_ptr = tuple.get_third();
    let third_str = unsafe { CStr::from_ptr(third_ptr).to_string_lossy() };

    println!("Tuple3: first={}, second={}, third={}", first, second, third_str);

    println!();

    // Tuple4 demo
    println!("--- Tuple4 (int, double, string, int) Demo ---");
    let third = CString::new("tuple").unwrap();
    let tuple = unsafe { tuple4_new(1, 2.71828, third.as_ptr(), 4) };

    println!("Tuple4 elements:");
    println!("  [0] = {}", tuple.get_first());
    println!("  [1] = {}", tuple.get_second());
    let third_ptr = tuple.get_third();
    let third_str = unsafe { CStr::from_ptr(third_ptr).to_string_lossy() };
    println!("  [2] = {}", third_str);
    println!("  [3] = {}", tuple.get_fourth());

    println!();

    // Using helper functions
    println!("--- Helper Functions Demo ---");
    let second = CString::new("pair").unwrap();
    let pair = unsafe { make_int_string_pair(10, second.as_ptr()) };
    let first = pair.get_first();
    let second_ptr = pair.get_second();
    let second_str = unsafe { CStr::from_ptr(second_ptr).to_string_lossy() };
    println!("make_int_string_pair: ({}, {})", first, second_str);

    println!("\nRust FFI: std::tuple 映射");
    println!("1. std::tuple 是异构容器的编译时固定版本");
    println!("2. 通过 std::get<N>(tuple) 访问元素");
    println!("3. FFI 需要为每个元素类型提供独立的 getter 函数");
    println!("4. 字符串等复杂类型需要额外的内存管理");
}

