use map_basic::*;

fn main() {
    use std::ffi::CString;

    println!("=== 035_map_basic - std::map ===\n");

    // StringIntMap demo
    println!("--- StringIntMap Demo ---");
    let mut map = string_int_map_new();

    println!("Empty: {}", map.empty());

    // Insert key-value pairs
    let keys = ["one", "two", "three", "four", "five"];
    let values = [1, 2, 3, 4, 5];

    for i in 0..keys.len() {
        let key = CString::new(keys[i]).unwrap();
        let inserted = map.insert(key.as_ptr(), values[i]);
        println!("Insert '{}' = {}: {}", keys[i], values[i], inserted);
    }

    let size = map.size();
    println!("Size: {}", size);

    // Get value
    let key = CString::new("one").unwrap();
    let val = map.get(key.as_ptr());
    println!("Get 'one': {}", val);

    // Set value
    let key = CString::new("one").unwrap();
    map.set(key.as_ptr(), 100);
    let key = CString::new("one").unwrap();
    let val = map.get(key.as_ptr());
    println!("Set 'one' = 100, now: {}", val);

    // Erase
    let key = CString::new("five").unwrap();
    let erased = map.erase(key.as_ptr());
    println!("Erase 'five': {}", erased);
    println!("Size after erase: {}", map.size());

    map.clear();
    println!("After clear, size: {}", map.size());

    println!("\nRust FFI: std::map 映射");
    println!("1. map 是有序关联容器（红黑树实现）");
    println!("2. 插入: insert(key, value) -> bool");
    println!("3. 查找: find(key) -> iterator 或 end()");
    println!("4. 删除: erase(key) -> size_t");
    println!("5. 字符串键需要 CString 转换");
}

