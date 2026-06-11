use union_basic::*;

use hicc::AbiClass;

fn variant_type_name(type_code: i32) -> &'static str {
    match type_code {
        0 => "INT",
        1 => "FLOAT",
        2 => "STRING",
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
    unsafe { union_set_int(&union_ptr.as_mut_ptr(), 0x41414141); }  // 'AAAA' in ASCII
    let int_val = union_get_int(&union_ptr.as_mut_ptr());
    println!("Set as int: {} (0x{:08x})", int_val, int_val as u32);

    // Read same memory as float
    let float_bits = union_get_float(&union_ptr.as_mut_ptr());
    println!("Read as float: {} (bits: 0x{:08x})", float_bits, float_bits.to_bits());

    println!("\n--- Summary ---");
    println!("1. union all members share the same memory");
    println!("2. Modifying one member affects other members");
    println!("3. union size equals the largest member size");
    println!("4. Often used to save memory or for type punning");
    println!("5. FFI passes union via variant wrapper");
}
