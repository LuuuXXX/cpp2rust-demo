hicc::cpp! {
    #include <iostream>
    #include <cstring>

    #include "class_copy.h"

    std::unique_ptr<Buffer> _cpp2rust_make_unique_buffer_0() { return std::make_unique<Buffer>(); }
    std::unique_ptr<Buffer> _cpp2rust_make_unique_buffer_with_sz(int sz) { return std::make_unique<Buffer>(sz); }
}

hicc::import_class! {
    #[cpp(class = "Buffer")]
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

    #[cpp(func = "std::unique_ptr<Buffer> _cpp2rust_make_unique_buffer_0()")]
    pub fn buffer_new() -> Buffer;

    #[cpp(func = "std::unique_ptr<Buffer> _cpp2rust_make_unique_buffer_with_sz(int)")]
    pub fn buffer_new_with_sz(sz: i32) -> Buffer;
}
