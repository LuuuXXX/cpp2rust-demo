hicc::cpp! {
    #include <iostream>
    #include <cstring>

    class Buffer {
        int* data;
        int size;
    public:
        Buffer() : data(nullptr), size(0) {}
        Buffer(int n) : data(new int[n]()), size(n) {}
        Buffer(const Buffer& other) : size(other.size) {
    data = new int[other.size];
    std::memcpy(data, other.data, other.size * sizeof(int));
}
        Buffer& operator=(const Buffer& other) {
    if (this != &other) {
        delete[] data;
        size = other.size;
        data = new int[size];
        std::memcpy(data, other.data, size * sizeof(int));
    }
    return *this;
}
        ~Buffer() {
    delete[] data;
}
        void set(int index, int value) {
    if (index >= 0 && index < size) {
        data[index] = value;
    }
}
        int get(int index) const {
    if (index >= 0 && index < size) {
        return data[index];
    }
    return 0;
}
        int getSize() const {
    return size;
}
    };

    Buffer* buffer_new() {
        return new Buffer();
    }

    Buffer* buffer_newWithSize(int size) {
        return new Buffer(size);
    }

    Buffer* buffer_newCopy(const Buffer* other) {
        return new Buffer(*other);
    }

    void buffer_delete(Buffer* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "Buffer")]
    class Buffer {
        #[cpp(method = "void set(int index, int value)")]
        fn set(&mut self, index: i32, value: i32);

        #[cpp(method = "int get(int index) const")]
        fn get(&self, index: i32) -> i32;

        #[cpp(method = "int getSize() const")]
        fn get_size(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "class_copy"]

    class Buffer;

    #[cpp(func = "Buffer* buffer_new()")]
    fn buffer_new() -> *mut Buffer;

    #[cpp(func = "Buffer* buffer_newWithSize(int)")]
    fn buffer_new_with_size(size: i32) -> *mut Buffer;

    #[cpp(func = "Buffer* buffer_newCopy(const struct Buffer* other)")]
    fn buffer_new_copy(other: *const Buffer) -> *mut Buffer;

    #[cpp(func = "void buffer_delete(Buffer* self)")]
    unsafe fn buffer_delete(self_: *mut Buffer);
}

fn main() {
    unsafe {
        // Create buffer
        let mut buf1 = buffer_new_with_size(5);
        println!("buf1 size: {}", buf1.get_size());

        // Set values
        for i in 0..5 {
            buf1.set(i, (i + 1) * 10);
        }

        // Get values
        print!("buf1 values: ");
        for i in 0..5 {
            print!("{} ", buf1.get(i));
        }
        println!();

        // Copy constructor
        let buf1_const = buf1 as *const Buffer;
        let buf2 = buffer_new_copy(&buf1_const);
        println!("buf2 created by copy");
        println!("buf2 size: {}", buf2.get_size());

        print!("buf2 values: ");
        for i in 0..5 {
            print!("{} ", buf2.get(i));
        }
        println!();

        // Modifying original does not affect copy
        buf1.set(0, 999);
        println!("After modifying buf1[0] = 999:");
        println!("buf1[0] = {}", buf1.get(0));
        println!("buf2[0] = {} (unchanged)", buf2.get(0));

        // Cleanup
        buffer_delete(&buf1);
        buffer_delete(&buf2);
    }

    println!("\nRust FFI: Copy constructor pattern works!");
}



