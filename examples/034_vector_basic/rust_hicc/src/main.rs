hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <vector>
    #include <string>
    #include <cstring>

    class IntVectorImpl {
    public:
        std::vector<int> data;
    public:
        IntVectorImpl() : data() {
}
        ~IntVectorImpl() {
    data.clear();
}
    };

    class StringVectorImpl {
    public:
        std::vector<std::string> data;
    public:
        StringVectorImpl() : data() {
}
        ~StringVectorImpl() {
    data.clear();
}
    };

    struct IntVector {
    public:
        IntVectorImpl* impl;
        IntVector() : impl(new IntVectorImpl()) {
}
        ~IntVector() {
    delete impl;
    impl = nullptr;
}
        void push_back(int val) { impl->data.push_back(val); }
        int get(size_t i) const { return impl->data[i]; }
        void set(size_t i, int val) { impl->data[i] = val; }
        size_t size() const { return impl->data.size(); }
        bool empty() const { return impl->data.empty(); }
        size_t capacity() const { return impl->data.capacity(); }
        int* data() { return impl->data.data(); }
        void clear() { impl->data.clear(); }
    };

    struct StringVector {
    public:
        StringVectorImpl* impl;
        StringVector() : impl(new StringVectorImpl()) {
}
        ~StringVector() {
    delete impl;
    impl = nullptr;
}
        size_t size() const { return impl->data.size(); }
    };

    IntVector* int_vector_new() {
        return new IntVector();
    }

    void int_vector_delete(IntVector* self) {
        delete self;
    }

    StringVector* string_vector_new() {
        return new StringVector();
    }

    void string_vector_delete(StringVector* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "IntVector")]
    class IntVector {
        #[cpp(method = "void push_back(int val)")]
        fn push_back(&mut self, val: i32);

        #[cpp(method = "int get(size_t i) const")]
        fn get(&self, i: usize) -> i32;

        #[cpp(method = "void set(size_t i, int val)")]
        fn set(&mut self, i: usize, val: i32);

        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "size_t capacity() const")]
        fn capacity(&self) -> usize;

        #[cpp(method = "int* data()")]
        fn data(&mut self) -> *mut i32;

        #[cpp(method = "void clear()")]
        fn clear(&mut self);
    }
}

hicc::import_class! {
    #[cpp(class = "StringVector")]
    class StringVector {
        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;
    }
}

hicc::import_lib! {
    #![link_name = "vector_basic"]

    class IntVector;
    class StringVector;

    #[cpp(func = "IntVector* int_vector_new()")]
    fn int_vector_new() -> *mut IntVector;

    #[cpp(func = "void int_vector_delete(IntVector* self)")]
    unsafe fn int_vector_delete(self_: *mut IntVector);

    #[cpp(func = "StringVector* string_vector_new()")]
    fn string_vector_new() -> *mut StringVector;

    #[cpp(func = "void string_vector_delete(StringVector* self)")]
    unsafe fn string_vector_delete(self_: *mut StringVector);
}

fn main() {
    println!("=== 034_vector_basic - std::vector ===\n");

    // IntVector demo
    println!("--- IntVector Demo ---");
    let mut vec = int_vector_new();

    println!("Empty: {}", vec.empty());

    // Push elements
    for i in 0..5 {
        vec.push_back((i * 10) as i32);
    }

    let size = vec.size();
    let capacity = vec.capacity();
    println!("Size: {}, Capacity: {}", size, capacity);

    // Access elements
    println!("Elements:");
    for i in 0..size {
        let val = vec.get(i);
        println!("  [{}] = {}", i, val);
    }

    // Modify element
    vec.set(2, 999);
    println!("After set [2] = 999: {}", vec.get(2));

    // Get raw data pointer
    let data_ptr = vec.data();
    println!("Raw data pointer: {:?}", data_ptr);

    vec.clear();
    println!("After clear, size: {}", vec.size());

    unsafe {
        int_vector_delete(&vec);
    }

    println!("\nRust FFI: std::vector 映射");
    println!("1. Opaque 指针隐藏 vector 内部结构");
    println!("2. push_back/get/set 等价于 Rust 的 push/get/index");
    println!("3. size()/capacity() 提供容器信息");
    println!("4. data() 获取原始指针用于批量操作");
    println!("\nNote: StringVector example omitted due to FFI complexity with const char*");
}



