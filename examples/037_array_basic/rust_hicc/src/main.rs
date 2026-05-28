hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <array>
    #include <string>
    #include <cstring>

    class IntArray5Impl {
    public:
        std::array<int, 5> data;
    public:
        IntArray5Impl() : data() {
}
        IntArray5Impl(const int* values) : data() {
    if (values) {
        for (size_t i = 0; i < 5; ++i) {
            data[i] = values[i];
        }
    }
}
        ~IntArray5Impl() {
}
    };

    class DoubleArray3Impl {
    public:
        std::array<double, 3> data;
    public:
        DoubleArray3Impl() : data() {
}
        DoubleArray3Impl(const double* values) : data() {
    if (values) {
        for (size_t i = 0; i < 3; ++i) {
            data[i] = values[i];
        }
    }
}
        ~DoubleArray3Impl() {
}
    };

    class StringArray4Impl {
    public:
        std::array<std::string, 4> data;
        bool initialized[4];
    public:
        StringArray4Impl() : data(), initialized{false, false, false, false} {
}
        ~StringArray4Impl() {
}
    };

    struct IntArray5 {
    public:
        IntArray5Impl* impl;
        IntArray5() : impl(new IntArray5Impl()) {
}
        IntArray5(const int* values) : impl(new IntArray5Impl(values)) {
}
        ~IntArray5() {
    delete impl;
    impl = nullptr;
}
        size_t size() const { return impl->data.size(); }
        bool empty() const { return impl->data.empty(); }
        void set(size_t i, int val) { impl->data[i] = val; }
        int get(size_t i) const { return impl->data[i]; }
        int at(size_t i) const { return impl->data.at(i); }
        int* data() { return impl->data.data(); }
    };

    struct DoubleArray3 {
    public:
        DoubleArray3Impl* impl;
        DoubleArray3() : impl(new DoubleArray3Impl()) {
}
        DoubleArray3(const double* values) : impl(new DoubleArray3Impl(values)) {
}
        ~DoubleArray3() {
    delete impl;
    impl = nullptr;
}
        size_t size() const { return impl->data.size(); }
    };

    struct StringArray4 {
    public:
        StringArray4Impl* impl;
        StringArray4() : impl(new StringArray4Impl()) {
}
        ~StringArray4() {
    delete impl;
    impl = nullptr;
}
        size_t size() const { return impl->data.size(); }
    };

    IntArray5* int_array5_new() {
        return new IntArray5();
    }

    IntArray5* int_array5_new_from(const int* values) {
        return new IntArray5(values);
    }

    void int_array5_delete(IntArray5* self) {
        delete self;
    }

    DoubleArray3* double_array3_new() {
        return new DoubleArray3();
    }

    DoubleArray3* double_array3_new_from(const double* values) {
        return new DoubleArray3(values);
    }

    void double_array3_delete(DoubleArray3* self) {
        delete self;
    }

    StringArray4* string_array4_new() {
        return new StringArray4();
    }

    void string_array4_delete(StringArray4* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "IntArray5")]
    class IntArray5 {
        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "void set(size_t, int)")]
        fn set(&mut self, i: usize, val: i32);

        #[cpp(method = "int get(size_t) const")]
        fn get(&self, i: usize) -> i32;

        #[cpp(method = "int at(size_t) const")]
        fn at(&self, i: usize) -> i32;

        #[cpp(method = "int* data()")]
        fn data(&mut self) -> *mut i32;
    }
}

hicc::import_class! {
    #[cpp(class = "DoubleArray3")]
    class DoubleArray3 {
        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;
    }
}

hicc::import_class! {
    #[cpp(class = "StringArray4")]
    class StringArray4 {
        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;
    }
}

hicc::import_lib! {
    #![link_name = "array_basic"]

    class IntArray5;
    class DoubleArray3;
    class StringArray4;

    #[cpp(func = "IntArray5* int_array5_new()")]
    fn int_array5_new() -> *mut IntArray5;

    #[cpp(func = "IntArray5* int_array5_new_from(const int*)")]
    fn int_array5_new_from(values: *const i32) -> *mut IntArray5;

    #[cpp(func = "void int_array5_delete(IntArray5* self)")]
    unsafe fn int_array5_delete(self_: *mut IntArray5);

    #[cpp(func = "DoubleArray3* double_array3_new()")]
    fn double_array3_new() -> *mut DoubleArray3;

    #[cpp(func = "DoubleArray3* double_array3_new_from(const double*)")]
    fn double_array3_new_from(values: *const f64) -> *mut DoubleArray3;

    #[cpp(func = "void double_array3_delete(DoubleArray3* self)")]
    unsafe fn double_array3_delete(self_: *mut DoubleArray3);

    #[cpp(func = "StringArray4* string_array4_new()")]
    fn string_array4_new() -> *mut StringArray4;

    #[cpp(func = "void string_array4_delete(StringArray4* self)")]
    unsafe fn string_array4_delete(self_: *mut StringArray4);
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



