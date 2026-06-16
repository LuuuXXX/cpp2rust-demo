use tuple_basic::*;

fn main() {
    println!("=== 038_tuple_basic - std::tuple（hicc 直出）===\n");

    let name = std::ffi::CString::new("alice").expect("CString::new failed");
    let mut record = Record::new(42, 98.5, name.as_ptr());
    let record_name = unsafe { std::ffi::CStr::from_ptr(record.name()).to_string_lossy().into_owned() };
    println!("id={} score={} name={}", record.id(), record.score(), record_name);

    record.set_id(100);
    record.set_score(88.25);
    let record_name = unsafe { std::ffi::CStr::from_ptr(record.name()).to_string_lossy().into_owned() };
    println!("after set id={} score={} name={}", record.id(), record.score(), record_name);

    println!("\nRust FFI: hicc 直接绑定持有 std::tuple 的类，析构由 Rust Drop 自动完成");
}
