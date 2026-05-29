hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <cstring>
    #include <new>

    Buffer* buffer_new(size_t capacity) {
        return new Buffer(capacity);
    }

    void buffer_delete(Buffer* self) {
        if (self) {
            std::cout << "Buffer delete called" << std::endl;
            delete self;
        }
    }

    VectorBuffer* vector_buffer_new(size_t capacity) {
        return new VectorBuffer(capacity, sizeof(SimpleValue));
    }

    void vector_buffer_delete(VectorBuffer* self) {
        if (self) {
            self->destroy_all();
            delete self;
        }
    }
}

hicc::import_class! {
    #[cpp(class = "Buffer", destroy = "vector_buffer_delete")]
    class Buffer {
        #[cpp(method = "void* data()")]
        fn data(&mut self) -> *mut u8;

        #[cpp(method = "size_t capacity() const")]
        fn capacity(&self) -> usize;

        #[cpp(method = "size_t size() const")]
        fn size(&self) -> usize;

        #[cpp(method = "void* construct(size_t offset)")]
        fn construct(&mut self, offset: usize) -> *mut u8;
    }
}

hicc::import_class! {
    #[cpp(class = "VectorBuffer")]
    class VectorBuffer {
        #[cpp(method = "void* data()")]
        fn data(&mut self) -> *mut u8;

        #[cpp(method = "size_t element_size() const")]
        fn element_size(&self) -> usize;

        #[cpp(method = "void destroy_all()")]
        fn destroy_all(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "placement_new"]

    class Buffer;
    class VectorBuffer;

    #[cpp(func = "Buffer* buffer_new(size_t)")]
    fn buffer_new(capacity: usize) -> Buffer;

    #[cpp(func = "VectorBuffer* vector_buffer_new(size_t)")]
    fn vector_buffer_new(capacity: usize) -> *mut VectorBuffer;
}

fn main() {
    println!("=== 032_placement_new - Placement New ===\n");

    // 创建预分配缓冲区
    let capacity = 1024;
    let mut buffer = buffer_new(capacity);
    println!("Buffer created with capacity: {}", capacity);

    let data_ptr = buffer.data();
    println!("Buffer data at: {:?}", data_ptr);

    let buf_capacity = buffer.capacity();
    println!("Buffer capacity: {}", buf_capacity);

    let buf_size = buffer.size();
    println!("Buffer constructed size: {}", buf_size);

    unsafe { buffer_delete(&buffer) };

    println!("\n--- VectorBuffer Demo ---");

    // VectorBuffer 示例
    let mut vec_buffer = vector_buffer_new(10);
    let elem_size = vec_buffer.element_size();
    println!("VectorBuffer element size: {}", elem_size);

    unsafe { vector_buffer_delete(&vec_buffer) };

    println!("\nRust FFI: Placement New 模式");
    println!("1. 在预分配内存中构造对象");
    println!("2. 使用 placement new: new (address) Constructor(args)");
    println!("3. 适用于内存池、STL 容器实现");
    println!("4. Rust 需要手动管理内存布局");
}

