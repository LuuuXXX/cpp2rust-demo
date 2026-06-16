use custom_deleter::*;

fn main() {
    println!("=== 031_custom_deleter - 自定义删除器（hicc 直出）===\n");

    let before = cleanup_count();

    {
        let name = std::ffi::CString::new("logfile.txt").expect("CString::new failed");
        let mut res = ManagedResource::new(name.as_ptr());

        let nm = unsafe { std::ffi::CStr::from_ptr(res.name()).to_string_lossy().into_owned() };
        println!("name={} released={}", nm, res.released());

        res.release();
        println!("after release released={}", res.released());
    }

    println!("cleanup_count delta={}", cleanup_count() - before);

    println!("\nRust FFI: hicc 绑定内部使用 unique_ptr<T, Deleter> 的类，");
    println!("自定义删除器在对象析构（Rust Drop）时被自动调用");
}
