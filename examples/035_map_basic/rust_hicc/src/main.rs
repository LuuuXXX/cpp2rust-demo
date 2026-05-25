hicc::cpp! {
    #include <map>
    #include <string>

    template<typename K, typename V>
    class MapImpl {
    public:
        std::map<K, V> data;
        MapImpl() = default;
        ~MapImpl() { data.clear(); }
    };

    class StringIntMap {
    public:
        MapImpl<std::string, int>* impl;
        StringIntMap() : impl(new MapImpl<std::string, int>()) {}
        ~StringIntMap() { delete impl; }
        unsigned long size() const { return impl->data.size(); }
        bool empty() const { return impl->data.empty(); }
        bool insert(const char* key, int value) {
            if (!key) return false;
            auto result = impl->data.insert({std::string(key), value});
            return result.second;
        }
        bool erase(const char* key) {
            if (!key) return false;
            return impl->data.erase(std::string(key)) > 0;
        }
        void clear() { impl->data.clear(); }
        int get(const char* key) const {
            if (!key) return 0;
            auto it = impl->data.find(std::string(key));
            if (it != impl->data.end()) return it->second;
            return 0;
        }
        void set(const char* key, int value) {
            if (key) impl->data[std::string(key)] = value;
        }
    };

    StringIntMap* string_int_map_new() { return new StringIntMap(); }
    void string_int_map_delete(StringIntMap* self) { delete self; }
}

hicc::import_class! {
    #[cpp(class = "StringIntMap")]
    class StringIntMap {
        #[cpp(method = "unsigned long size() const")]
        fn size(&self) -> u64;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "bool insert(const char*, int)")]
        fn insert(&mut self, key: *const i8, value: i32) -> bool;

        #[cpp(method = "bool erase(const char*)")]
        fn erase(&mut self, key: *const i8) -> bool;

        #[cpp(method = "void clear()")]
        fn clear(&mut self);

        #[cpp(method = "int get(const char*) const")]
        fn get(&self, key: *const i8) -> i32;

        #[cpp(method = "void set(const char*, int)")]
        fn set(&mut self, key: *const i8, value: i32);
    }
}

hicc::import_lib! {
    #![link_name = "map_basic"]

    class StringIntMap;

    #[cpp(func = "StringIntMap* string_int_map_new()")]
    fn string_int_map_new() -> *mut StringIntMap;

    #[cpp(func = "void string_int_map_delete(StringIntMap* self)")]
    unsafe fn string_int_map_delete(self_: *mut StringIntMap);
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

    unsafe {
        string_int_map_delete(&map);
    }

    println!("\nRust FFI: std::map 映射");
    println!("1. map 是有序关联容器（红黑树实现）");
    println!("2. 插入: insert(key, value) -> bool");
    println!("3. 查找: find(key) -> iterator 或 end()");
    println!("4. 删除: erase(key) -> size_t");
    println!("5. 字符串键需要 CString 转换");
}
