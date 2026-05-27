hicc::cpp! {
    #include <string>
    #include <iostream>
    #include <memory>
    #include <cstring>

    class UniqueBuffer {
        std::string data;
    public:
        UniqueBuffer(int sz) : data(sz, '\0') {
}
        ~UniqueBuffer() {
}
        int getSize() const {
    return static_cast<int>(data.size());
}
        char* getData() {
    return data.data();
}
        UniqueBuffer move() {
    return UniqueBuffer(*this);
}
        int useCount() const {
    return 1; // unique_ptr always has use count of 1
}
    };

    class Processor {
        std::string buffer;
    public:
        Processor() : buffer() {
}
        ~Processor() {
}
        char* process(const char* input) {
    if (input) {
        buffer = std::string(input) + " [processed]";
    }
    return const_cast<char*>(buffer.c_str());
}
    };

    UniqueBuffer* uniquebuffer_new(int size) {
        return new UniqueBuffer(size);
    }

    void uniquebuffer_delete(UniqueBuffer* self) {
        delete self;
    }

    Processor* processor_new() {
        return new Processor();
    }

    void processor_delete(Processor* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "UniqueBuffer")]
    class UniqueBuffer {
        #[cpp(method = "int getSize() const")]
        fn get_size(&self) -> i32;

        #[cpp(method = "char* getData()")]
        fn get_data(&mut self) -> *mut u8;

        #[cpp(method = "UniqueBuffer move()")]
        fn move(&mut self) -> UniqueBuffer;

        #[cpp(method = "int useCount() const")]
        fn use_count(&self) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Processor")]
    class Processor {
        #[cpp(method = "char* process(const char* input)")]
        fn process(&mut self, input: *const u8) -> *mut u8;
    }
}

hicc::import_lib! {
    #![link_name = "unique_ptr"]

    class UniqueBuffer;
    class Processor;

    #[cpp(func = "UniqueBuffer* uniquebuffer_new(int)")]
    fn uniquebuffer_new(size: i32) -> *mut UniqueBuffer;

    #[cpp(func = "void uniquebuffer_delete(UniqueBuffer* self)")]
    unsafe fn uniquebuffer_delete(self_: *mut UniqueBuffer);

    #[cpp(func = "Processor* processor_new()")]
    fn processor_new() -> *mut Processor;

    #[cpp(func = "void processor_delete(Processor* self)")]
    unsafe fn processor_delete(self_: *mut Processor);
}

fn main() {
    println!("=== 029_unique_ptr - std::unique_ptr ===\n");

    // UniqueBuffer - 模拟 unique_ptr 自动内存管理
    let mut buffer = uniquebuffer_new(16);
    let size = buffer.get_size();
    println!("Buffer size: {}", size);

    let data_ptr = buffer.get_data();
    let slice = unsafe { std::slice::from_raw_parts(data_ptr as *const u8, size as usize) };
    let data_str: String = slice.iter().map(|&c| c as char).collect();
    println!("Buffer data: {}", data_str);

    let count = buffer.use_count();
    println!("Use count: {} (unique_ptr always = 1)", count);

    unsafe { uniquebuffer_delete(&buffer) };

    println!();

    // Processor - 内部使用 unique_ptr 管理资源
    let mut processor = processor_new();
    let input = std::ffi::CString::new("Hello, unique_ptr!").expect("CString::new failed");
    let result_ptr = processor.process(input.as_ptr());
    let result = unsafe { std::ffi::CStr::from_ptr(result_ptr).to_string_lossy().into_owned() };
    println!("Processed result: {}", result);
    unsafe { processor_delete(&processor) };

    println!("\nRust FFI: unique_ptr 的处理方式");
    println!("1. C++ 侧管理对象生命周期");
    println!("2. Rust 侧通过 FFI 函数调用管理");
    println!("3. 相当于 Rust 的 Box<T>");

    println!("\nhicc-std 提供了 std::unique_ptr 的安全 Rust 包装");
}


