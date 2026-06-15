hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <cstring>
    #include <new>

    #include "placement_new.h"

    std::unique_ptr<Buffer> _cpp2rust_make_unique_buffer_with_capacity(size_t capacity) { return std::make_unique<Buffer>(capacity); }
    std::unique_ptr<VectorBuffer> _cpp2rust_make_unique_vector_buffer_2(size_t capacity, size_t elem_size) { return std::make_unique<VectorBuffer>(capacity, elem_size); }
}

hicc::import_class! {
    #[cpp(class = "Buffer")]
    pub class Buffer {
        #[cpp(method = "void* data()")]
        pub fn data(&mut self) -> *mut u8;

        #[cpp(method = "size_t capacity() const")]
        pub fn capacity(&self) -> usize;

        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        #[cpp(method = "void* construct(size_t offset)")]
        pub fn construct(&mut self, offset: usize) -> *mut u8;
    }
}

hicc::import_class! {
    #[cpp(class = "VectorBuffer")]
    pub class VectorBuffer {
        #[cpp(method = "void* data()")]
        pub fn data(&mut self) -> *mut u8;

        #[cpp(method = "size_t element_size() const")]
        pub fn element_size(&self) -> usize;

        #[cpp(method = "void destroy_all()")]
        pub fn destroy_all(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "placement_new"]

    class Buffer;
    class VectorBuffer;

    #[cpp(func = "std::unique_ptr<Buffer> _cpp2rust_make_unique_buffer_with_capacity(size_t)")]
    pub fn buffer_new_with_capacity(capacity: usize) -> Buffer;

    #[cpp(func = "std::unique_ptr<VectorBuffer> _cpp2rust_make_unique_vector_buffer_2(size_t, size_t)")]
    pub fn vector_buffer_new_2(capacity: usize, elem_size: usize) -> VectorBuffer;
}
