use vector_basic::*;

fn main() {
    println!("=== 034_vector_basic - std::vector（hicc 直出）===\n");

    let mut v = IntVector::new();
    println!("empty={}", v.empty());
    v.reserve(8);
    for i in 0..5 {
        v.push_back(i * 10);
    }
    println!("size={} sum={}", v.size(), v.sum());
    v.set(2, 999);
    println!("get(2)={}", v.get(2));
    v.pop_back();
    println!("after pop_back size={}", v.size());
    v.clear();
    println!("after clear empty={}", v.empty());

    println!();

    let mut sv = StringVector::new();
    for s in ["alpha", "beta"] {
        let cs = std::ffi::CString::new(s).expect("CString::new failed");
        sv.push_back(cs.as_ptr());
    }
    let g0 = unsafe { std::ffi::CStr::from_ptr(sv.get(0)).to_string_lossy().into_owned() };
    let g1 = unsafe { std::ffi::CStr::from_ptr(sv.get(1)).to_string_lossy().into_owned() };
    println!("sv size={} get(0)={} get(1)={}", sv.size(), g0, g1);

    println!("\nRust FFI: hicc 直接绑定持有 std::vector 的类，析构由 Rust Drop 自动完成");
}
