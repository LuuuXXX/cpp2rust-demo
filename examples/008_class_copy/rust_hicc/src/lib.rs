hicc::cpp! {
    #include <iostream>
    #include <cstring>

    #include "class_copy.h"
}

hicc::import_class! {
    #[cpp(class = "Buffer", destroy = "buffer_delete")]
    pub class Buffer {
        #[cpp(method = "void set(int index, int value)")]
        pub fn set(&mut self, index: i32, value: i32);

        #[cpp(method = "int get(int index) const")]
        pub fn get(&self, index: i32) -> i32;

        #[cpp(method = "int getSize() const")]
        pub fn get_size(&self) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "class_copy"]

    class Buffer;

    #[cpp(func = "Buffer* buffer_new()")]
    pub fn buffer_new() -> Buffer;

    #[cpp(func = "Buffer* buffer_newWithSize(int)")]
    pub fn buffer_new_with_size(size: i32) -> Buffer;

    #[cpp(func = "Buffer* buffer_newCopy(const Buffer* other)")]
    pub fn buffer_new_copy(other: *const Buffer) -> Buffer;
}
