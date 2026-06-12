hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <cstring>
    #include <new>

    #include "placement_new.h"
}

hicc::import_class! {
    #[cpp(class = "Buffer", destroy = "buffer_delete")]
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
    #[cpp(class = "VectorBuffer", destroy = "vector_buffer_delete")]
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

    #[cpp(func = "Buffer* buffer_new(size_t)")]
    pub fn buffer_new(capacity: usize) -> Buffer;

    #[cpp(func = "VectorBuffer* vector_buffer_new(size_t)")]
    pub fn vector_buffer_new(capacity: usize) -> VectorBuffer;
}
