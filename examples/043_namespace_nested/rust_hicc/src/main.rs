use namespace_nested::*;
use std::ffi::{CStr, CString};

fn main() {
    println!("=== 043_namespace_nested - 嵌套命名空间（hicc 直出）===\n");

    let mut config = ConfigManager::new();
    for (key, value) in [("timeout", 30), ("retry", 3), ("port", 8080)] {
        let ck = CString::new(key).expect("CString::new failed");
        config.set_value(ck.as_ptr(), value);
    }
    let timeout = CString::new("timeout").expect("CString::new failed");
    let retry = CString::new("retry").expect("CString::new failed");
    let missing = CString::new("missing").expect("CString::new failed");
    println!(
        "size={} timeout={} retry={}",
        config.size(),
        config.get_value(timeout.as_ptr()),
        config.get_value(retry.as_ptr())
    );
    println!("missing={}", config.get_value(missing.as_ptr()));

    let processor = DataProcessor::new();
    println!("process(5)={}", processor.process(5));

    let version = unsafe { CStr::from_ptr(get_version()).to_string_lossy().into_owned() };
    println!("version={} build_number={}", version, get_build_number());

    println!("\nRust FFI: hicc 直接绑定嵌套命名空间类，析构由 Rust Drop 自动完成");
}
