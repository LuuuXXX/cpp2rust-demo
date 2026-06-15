// 此文件为 cpp2rust-demo 工具对 032_placement_new 自动生成的支架黄金文件，
// 仅供 L1 golden 测试（test_032_placement_new）校验工具默认产物的生成准确性。

hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <cstring>
    #include <new>

    #include "placement_new.h"
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

    #[cpp(func = "std::unique_ptr<Buffer> std::make_unique<Buffer>(size_t)")]
    pub fn buffer_new_with_capacity(capacity: usize) -> Buffer;

    #[cpp(func = "std::unique_ptr<VectorBuffer> std::make_unique<VectorBuffer>(size_t, size_t)")]
    pub fn vector_buffer_new_2(capacity: usize, elem_size: usize) -> VectorBuffer;
}
