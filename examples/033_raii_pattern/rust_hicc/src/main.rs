use raii_pattern::*;

fn main() {
    println!("=== 033_raii_pattern - RAII 资源管理（hicc 直出）===\n");

    println!("active_count(start)={}", active_count());
    {
        let name = std::ffi::CString::new("db").expect("CString::new failed");
        let a = Resource::new(name.as_ptr());
        let nm = unsafe { std::ffi::CStr::from_ptr(a.name()).to_string_lossy().into_owned() };
        println!("after acquire a: name={} active={}", nm, active_count());
        {
            let bn = std::ffi::CString::new("file").expect("CString::new failed");
            let _b = Resource::new(bn.as_ptr());
            println!("after acquire b: active={}", active_count());
        }
        println!("after b released: active={}", active_count());
    }
    println!("after all released: active={}", active_count());

    println!();

    let rb = rollback_count();
    {
        let mut t1 = Transaction::new();
        t1.commit();
        println!("t1 committed={}", t1.committed());
    }
    {
        let t2 = Transaction::new(); // 未提交 → Drop 时回滚
        println!("t2 committed={}", t2.committed());
    }
    println!("rollback delta={}", rollback_count() - rb);

    println!("\nRust FFI: hicc 绑定 RAII 类，构造获取资源、Drop 自动释放");
}
