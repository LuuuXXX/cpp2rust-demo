hicc::cpp! {
    #include <array>

    template<typename T, unsigned long N>
    class ArrayImpl {
    public:
        std::array<T, N> data;
        ArrayImpl() : data() {}
        explicit ArrayImpl(const T* values) {
            if (values) {
                for (unsigned long i = 0; i < N; ++i) {
                    data[i] = values[i];
                }
            }
        }
    };

    class IntArray5 {
    public:
        ArrayImpl<int, 5>* impl;
        IntArray5() : impl(new ArrayImpl<int, 5>()) {}
        explicit IntArray5(const int* values) : impl(new ArrayImpl<int, 5>(values)) {}
        ~IntArray5() { delete impl; }
        unsigned long size() const { return 5; }
        bool empty() const { return false; }
        int get(unsigned long index) const { return index < 5 ? impl->data[index] : 0; }
        void set(unsigned long index, int value) { if (index < 5) impl->data[index] = value; }
        int* data() { return impl->data.data(); }
        int at(unsigned long index) const { return impl->data.at(index); }
    };

    IntArray5* int_array5_new() { return new IntArray5(); }
    IntArray5* int_array5_new_from(const int* values) { return new IntArray5(values); }
    void int_array5_delete(IntArray5* self) { delete self; }
}

hicc::import_class! {
    #[cpp(class = "IntArray5")]
    class IntArray5 {
        #[cpp(method = "unsigned long size() const")]
        fn size(&self) -> u64;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "int get(unsigned long) const")]
        fn get(&self, index: u64) -> i32;

        #[cpp(method = "void set(unsigned long, int)")]
        fn set(&mut self, index: u64, value: i32);

        #[cpp(method = "int* data()")]
        fn data(&mut self) -> *mut i32;

        #[cpp(method = "int at(unsigned long) const")]
        fn at(&self, index: u64) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "array_basic"]

    class IntArray5;

    #[cpp(func = "IntArray5* int_array5_new()")]
    fn int_array5_new() -> *mut IntArray5;

    #[cpp(func = "IntArray5* int_array5_new_from(const int* values)")]
    fn int_array5_new_from(values: *const i32) -> *mut IntArray5;

    #[cpp(func = "void int_array5_delete(IntArray5* self)")]
    unsafe fn int_array5_delete(self_: *mut IntArray5);
}

fn main() {
    println!("=== 037_array_basic - std::array ===\n");

    // IntArray5 demo
    println!("--- IntArray5 Demo ---");
    let mut arr = int_array5_new();

    println!("Size: {}", arr.size());
    println!("Empty: {}", arr.empty());

    // Set elements
    for i in 0..5 {
        arr.set(i, (i * 10) as i32);
    }

    // Access elements
    println!("Elements:");
    for i in 0..5 {
        let val = arr.get(i);
        println!("  [{}] = {}", i, val);
    }

    // at() access
    let val = arr.at(2);
    println!("at(2) = {}", val);

    // data() pointer
    let data_ptr = arr.data();
    println!("Data pointer: {:?}", data_ptr);

    unsafe { int_array5_delete(&arr); }

    println!();

    // IntArray5 from values
    println!("--- IntArray5 from values Demo ---");
    let values = [1, 2, 3, 4, 5];
    let arr = int_array5_new_from(values.as_ptr());

    println!("Size: {}", arr.size());
    println!("Elements:");
    for i in 0..5 {
        let val = arr.get(i);
        println!("  [{}] = {}", i, val);
    }

    unsafe { int_array5_delete(&arr); }

    println!("\nRust FFI: std::array 映射");
    println!("1. std::array 是固定大小的数组容器");
    println!("2. 大小在编译时确定（模板参数）");
    println!("3. data() 返回原始指针用于批量访问");
    println!("4. 与 Rust 的 [T; N] 数组语义相似");
}
