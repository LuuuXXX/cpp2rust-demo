hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <cstring>
    #include <new>

    #include "placement_new.h"
}

use hicc::AbiClass;

hicc::import_class! {
    #[cpp(class = "Buffer", destroy = "buffer_delete")]
    pub class Buffer {
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
    #[cpp(class = "VectorBuffer", destroy = "vector_buffer_delete")]
    pub class VectorBuffer {
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
    fn vector_buffer_new(capacity: usize) -> VectorBuffer;
}

fn main() {
    println!("=== 032_placement_new - Placement New ===\n");

    // 创建预分配缓冲区
    let capacity = 1024;
    let mut buffer = unsafe { buffer_new(capacity).into_unique() };
    println!("Buffer created with capacity: {}", capacity);

    let data_ptr = buffer.data();
    println!("Buffer data at: {:?}", data_ptr);

    let buf_capacity = buffer.capacity();
    println!("Buffer capacity: {}", buf_capacity);

    let buf_size = buffer.size();
    println!("Buffer constructed size: {}", buf_size);

    drop(buffer);

    println!("\n--- VectorBuffer Demo ---");

    // VectorBuffer 示例
    let mut vec_buffer = vector_buffer_new(10);
    let elem_size = vec_buffer.element_size();
    println!("VectorBuffer element size: {}", elem_size);

    println!("\nRust FFI: Placement New 模式");
    println!("1. 在预分配内存中构造对象");
    println!("2. 使用 placement new: new (address) Constructor(args)");
    println!("3. 适用于内存池、STL 容器实现");
    println!("4. Rust 需要手动管理内存布局");
}

