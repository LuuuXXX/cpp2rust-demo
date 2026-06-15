use union_basic::*;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    println!("=== 045_union_basic - Unions ===\n");

    println!("--- Variant Demo ---");

    let v_int = variant_new_int(42);
    println!("Type: {}, Value: {}", variant_type_name(v_int.get_type()), v_int.get_int());

    let v_float = variant_new_float(3.14);
    println!("Type: {}, Value: {}", variant_type_name(v_float.get_type()), v_float.get_float());

    let v_string = unsafe { variant_new_string("Hello, Union!\0".as_ptr() as *const i8) };
    println!("Type: {}, Value: {}", variant_type_name(v_string.get_type()), decode_cstr(v_string.get_string()));

    println!("\n--- Memory Overlay Demo ---");
    println!("sizeof(int) = {}, sizeof(float) = {}", std::mem::size_of::<i32>(), std::mem::size_of::<f32>());

    let mut union_ptr = union_new();

    union_ptr.set_int(0x41414141);
    let int_val = union_ptr.get_int();
    println!("Set as int: {} (0x{:08x})", int_val, int_val as u32);

    let float_bits = union_ptr.get_float().to_bits();
    println!("Read as float: {} (bits: 0x{:08x})", union_ptr.get_float(), float_bits);

    println!("\n--- Summary ---");
    println!("1. union all members share the same memory");
    println!("2. Modifying one member affects other members");
    println!("3. union size equals the largest member size");
    println!("4. Often used to save memory or for type punning");
    println!("5. FFI passes union via variant wrapper");
}
