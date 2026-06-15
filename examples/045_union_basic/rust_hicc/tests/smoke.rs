use union_basic::*;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

#[test]
fn smoke_variant_int() {
    let v = variant_new_int(42);
    assert_eq!(v.get_type(), 0, "Variant INT type code should be 0");
    assert_eq!(v.get_int(), 42, "Variant int value should be 42");
}

#[test]
fn smoke_variant_float() {
    let v = variant_new_float(3.14);
    assert_eq!(v.get_type(), 1, "Variant FLOAT type code should be 1");
    assert!((v.get_float() - 3.14f32).abs() < 0.01, "Variant float value should be close to 3.14");
}

#[test]
fn smoke_variant_string() {
    let v = unsafe { variant_new_string("hello\0".as_ptr() as *const i8) };
    assert_eq!(v.get_type(), 2, "Variant STRING type code should be 2");
    assert_eq!(decode_cstr(v.get_string()), "hello", "Variant string value should be hello");
}

#[test]
fn smoke_union_memory_overlay() {
    let mut u = union_new();
    u.set_int(0x41414141);
    assert_eq!(u.get_int(), 0x41414141i32, "reading int value should match written value");
    let float_bits = u.get_float().to_bits();
    assert_eq!(float_bits, 0x41414141u32, "float bits representation should match int (union shared memory)");
}

#[test]
fn smoke_variant_type_name() {
    assert_eq!(variant_type_name(0), "INT");
    assert_eq!(variant_type_name(1), "FLOAT");
    assert_eq!(variant_type_name(2), "STRING");
    assert_eq!(variant_type_name(99), "Unknown");
}
