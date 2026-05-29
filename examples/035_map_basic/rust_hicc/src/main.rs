hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <map>
    #include <string>
    #include <cstring>

    StringIntMap* string_int_map_new(void) {
        return new StringIntMap();
    }

    void string_int_map_delete(StringIntMap* self) {
        delete self;
    }

    IntStringMap* int_string_map_new(void) {
        return new IntStringMap();
    }

    void int_string_map_delete(IntStringMap* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "StringIntMap", destroy = "string_int_map_delete")]
    class StringIntMap {
        #[cpp(method = "bool insert(const char* key, int val)")]
        fn insert(&mut self, key: *const i8, val: i32) -> bool;

        #[cpp(method = "int get(const char* key) const")]
        fn get(&self, key: *const i8) -> i32;

        #[cpp(method = "void set(const char* key, int val)")]
        fn set(&mut self, key: *const i8, val: i32);

        #[cpp(method = "bool erase(const char* key)")]
        fn erase(&mut self, key: *const i8) -> bool;

        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "void clear()")]
        fn clear(&mut self);
    }
}

hicc::import_class! {
    #[cpp(class = "IntStringMap", destroy = "int_string_map_delete")]
    class IntStringMap {
        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;
    }
}

hicc::import_lib! {
    #![link_name = "map_basic"]

    class StringIntMap;
    class IntStringMap;

    #[cpp(func = "StringIntMap* string_int_map_new()")]
    fn string_int_map_new() -> StringIntMap;

    #[cpp(func = "IntStringMap* int_string_map_new()")]
    fn int_string_map_new() -> IntStringMap;
}

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

