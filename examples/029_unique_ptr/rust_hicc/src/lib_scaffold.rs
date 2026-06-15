// 此文件为 cpp2rust-demo 工具对 029_unique_ptr 自动生成的支架黄金文件，
// 仅供 L1 golden 测试（test_029_unique_ptr）校验工具默认产物的生成准确性。

hicc::cpp! {
    #include <string>
    #include <iostream>
    #include <memory>
    #include <cstring>

    #include "unique_ptr.h"
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

    #[cpp(func = "std::unique_ptr<UniqueBuffer> std::make_unique<UniqueBuffer>(int)")]
    pub fn unique_buffer_new_with_sz(sz: i32) -> UniqueBuffer;

    #[cpp(func = "std::unique_ptr<Processor> hicc::make_unique<Processor>()")]
    pub fn processor_new() -> Processor;
}
