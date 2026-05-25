hicc::cpp! {
    #include <vector>

    template<typename T>
    class VectorImpl {
    public:
        std::vector<T> data;
        VectorImpl() = default;
        ~VectorImpl() { data.clear(); }
    };

    class IntVector {
    public:
        VectorImpl<int>* impl;
        IntVector() : impl(new VectorImpl<int>()) {}
        ~IntVector() { delete impl; }
        unsigned long size() const { return impl->data.size(); }
        unsigned long capacity() const { return impl->data.capacity(); }
        bool empty() const { return impl->data.empty(); }
        void push_back(int value) { impl->data.push_back(value); }
        int get(unsigned long index) const { return index < impl->data.size() ? impl->data[index] : 0; }
        void set(unsigned long index, int value) { if (index < impl->data.size()) impl->data[index] = value; }
        void clear() { impl->data.clear(); }
        int* data() { return impl->data.data(); }
    };

    IntVector* int_vector_new() { return new IntVector(); }
    void int_vector_delete(IntVector* self) { delete self; }
}

hicc::import_class! {
    #[cpp(class = "IntVector")]
    class IntVector {
        #[cpp(method = "unsigned long size() const")]
        fn size(&self) -> u64;

        #[cpp(method = "unsigned long capacity() const")]
        fn capacity(&self) -> u64;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "void push_back(int)")]
        fn push_back(&mut self, value: i32);

        #[cpp(method = "int get(unsigned long) const")]
        fn get(&self, index: u64) -> i32;

        #[cpp(method = "void set(unsigned long, int)")]
        fn set(&mut self, index: u64, value: i32);

        #[cpp(method = "int* data()")]
        fn data(&mut self) -> *mut i32;

        #[cpp(method = "void clear()")]
        fn clear(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "vector_basic"]

    class IntVector;

    #[cpp(func = "IntVector* int_vector_new()")]
    fn int_vector_new() -> *mut IntVector;

    #[cpp(func = "void int_vector_delete(IntVector* self)")]
    unsafe fn int_vector_delete(self_: *mut IntVector);
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
