hicc::cpp! {
    #include <string>
    #include <iostream>
    #include <memory>
    #include <cstring>

    #include "unique_ptr.h"

    std::unique_ptr<UniqueBuffer> _cpp2rust_make_unique_unique_buffer_with_sz(int sz) { return std::make_unique<UniqueBuffer>(sz); }
}

hicc::import_class! {
    #[cpp(class = "UniqueBuffer")]
    pub class UniqueBuffer {
        #[cpp(method = "int getSize() const")]
        pub fn get_size(&self) -> i32;

        #[cpp(method = "char* getData()")]
        pub fn get_data(&mut self) -> *mut i8;

        #[cpp(method = "int useCount() const")]
        pub fn use_count(&self) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Processor")]
    pub class Processor {
        #[cpp(method = "char* process(const char* input)")]
        pub fn process(&mut self, input: *const i8) -> *mut i8;
    }
}

hicc::import_lib! {
    #![link_name = "unique_ptr"]

    class UniqueBuffer;
    class Processor;

    #[cpp(func = "std::unique_ptr<UniqueBuffer> _cpp2rust_make_unique_unique_buffer_with_sz(int)")]
    pub fn unique_buffer_new_with_sz(sz: i32) -> UniqueBuffer;

    #[cpp(func = "std::unique_ptr<Processor> hicc::make_unique<Processor>()")]
    pub fn processor_new() -> Processor;
}
