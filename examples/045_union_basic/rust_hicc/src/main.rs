hicc::cpp! {
    #include <cstddef>
    #include <cstdint>
    #include <iostream>
    #include <cstring>

    #include "union_basic.h"
}

hicc::import_class! {
    #[cpp(class = "IntFloatUnion", destroy = "union_delete")]
    class IntFloatUnion {}
}

hicc::import_class! {
    #[cpp(class = "Variant", destroy = "variant_delete")]
    class Variant {
        #[cpp(method = "int get_type() const")]
        fn get_type(&self) -> i32;

        #[cpp(method = "void set_int(int value)")]
        fn set_int(&mut self, value: i32);

        #[cpp(method = "void set_float(float value)")]
        fn set_float(&mut self, value: f32);

        #[cpp(method = "void set_string(const char* value)")]
        fn set_string(&mut self, value: *const i8);

        #[cpp(method = "int get_int() const")]
        fn get_int(&self) -> i32;

        #[cpp(method = "float get_float() const")]
        fn get_float(&self) -> f32;

        #[cpp(method = "const char* get_string() const")]
        fn get_string(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "union_basic"]

    class IntFloatUnion;
    class Variant;

    #[cpp(func = "IntFloatUnion* union_new()")]
    fn union_new() -> IntFloatUnion;

    #[cpp(func = "Variant* variant_new_int(int)")]
    fn variant_new_int(value: i32) -> Variant;

    #[cpp(func = "Variant* variant_new_float(float)")]
    fn variant_new_float(value: f32) -> Variant;

    #[cpp(func = "Variant* variant_new_string(const char*)")]
    unsafe fn variant_new_string(value: *const i8) -> Variant;

    #[cpp(func = "int union_get_int(IntFloatUnion* u)")]
    fn union_get_int(u: *mut IntFloatUnion) -> i32;

    #[cpp(func = "float union_get_float(IntFloatUnion* u)")]
    fn union_get_float(u: *mut IntFloatUnion) -> f32;

    #[cpp(func = "void union_set_int(IntFloatUnion* u, int)")]
    unsafe fn union_set_int(u: *mut IntFloatUnion, value: i32);

    #[cpp(func = "void union_set_float(IntFloatUnion* u, float)")]
    unsafe fn union_set_float(u: *mut IntFloatUnion, value: f32);
}

fn variant_type_name(type_code: i32) -> &'static str {
    match type_code {
        0 => "Int",
        1 => "Float",
        2 => "String",
        _ => "Unknown",
    }
}

fn main() {
    println!("=== 045_union_basic - Unions ===\n");

    // Variant example
    println!("--- Variant Demo ---");

    let v_int = variant_new_int(42);
    println!("Type: {}, Value: {}", variant_type_name(v_int.get_type()), v_int.get_int());

    let v_float = variant_new_float(3.14);
    println!("Type: {}, Value: {}", variant_type_name(v_float.get_type()), v_float.get_float());

    let v_string = unsafe { variant_new_string("Hello, Union!\0".as_ptr() as *const i8) };
    let s = unsafe { std::ffi::CStr::from_ptr(v_string.get_string()) };
    println!("Type: {}, Value: {}", variant_type_name(v_string.get_type()), s.to_str().unwrap());

    // Memory overlay demo
    println!("\n--- Memory Overlay Demo ---");
    println!("sizeof(int) = {}, sizeof(float) = {}", std::mem::size_of::<i32>(), std::mem::size_of::<f32>());

    let mut union_ptr = union_new();

    // Set int value
    unsafe { union_set_int(&mut union_ptr, 0x41414141); }  // 'AAAA' in ASCII
    let int_val = union_get_int(&union_ptr);
    println!("Set as int: {} (0x{:08x})", int_val, int_val as u32);

    // Read same memory as float
    let float_bits = union_get_float(&union_ptr);
    println!("Read as float: {} (bits: 0x{:08x})", float_bits, float_bits.to_bits());

    unsafe { union_delete(&union_ptr); };

    println!("\n--- Summary ---");
    println!("1. union all members share the same memory");
    println!("2. Modifying one member affects other members");
    println!("3. union size equals the largest member size");
    println!("4. Often used to save memory or for type punning");
    println!("5. FFI passes union via variant wrapper");
}

