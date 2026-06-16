use map_basic::*;

fn main() {
    use std::ffi::{CStr, CString};

    println!("=== 035_map_basic - std::map / std::unordered_map（hicc 直出）===\n");

    let mut m = StringIntMap::new();
    for (key, value) in [("apple", 3), ("banana", 5), ("apple", 7)] {
        let ck = CString::new(key).expect("CString::new failed");
        m.insert(ck.as_ptr(), value);
    }
    let apple = CString::new("apple").expect("CString::new failed");
    let banana = CString::new("banana").expect("CString::new failed");
    let missing = CString::new("missing").expect("CString::new failed");
    let first_key = unsafe { CStr::from_ptr(m.first_key()).to_string_lossy().into_owned() };
    println!(
        "size={} apple={} banana?={}",
        m.size(),
        m.get(apple.as_ptr()),
        m.contains(banana.as_ptr())
    );
    println!("missing={} first_key={}", m.get(missing.as_ptr()), first_key);
    println!("erase banana={} size={}", m.erase(banana.as_ptr()), m.size());
    m.clear();
    println!("after clear size={}", m.size());

    println!();

    let mut c = Counter::new();
    for word in ["rust", "cpp", "rust"] {
        let cw = CString::new(word).expect("CString::new failed");
        c.add(cw.as_ptr());
    }
    let rust = CString::new("rust").expect("CString::new failed");
    let cpp = CString::new("cpp").expect("CString::new failed");
    let last = unsafe { CStr::from_ptr(c.last_word()).to_string_lossy().into_owned() };
    println!(
        "counter rust={} cpp={} unique={} last={}",
        c.count(rust.as_ptr()),
        c.count(cpp.as_ptr()),
        c.unique_words(),
        last
    );

    println!("\nRust FFI: hicc 直接绑定持有 std::map / std::unordered_map 的类，析构由 Rust Drop 自动完成");
}
