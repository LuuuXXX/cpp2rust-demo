//! 029_unique_ptr: 独占所有权（命名空间类 + hicc unique_ptr 自动管理）。
//!
//! hicc 直出用 `std::unique_ptr` 持有 `UniqueBuffer` / `Processor` 的所有权，析构由
//! Rust 的 `Drop` 自动完成，无需手写 `*_delete`。`make_unique` 工厂经 `new` 关联函数暴露。

hicc::cpp! {
    #include "unique_ptr.h"
}

hicc::import_class! {
    #[cpp(class = "unique_ptr_ns::UniqueBuffer")]
    pub class UniqueBuffer {
        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        #[cpp(method = "char* data()")]
        pub fn data(&mut self) -> *mut i8;

        #[cpp(method = "void fill(char c)")]
        pub fn fill(&mut self, c: i8);

        #[cpp(method = "char at(int i) const")]
        pub fn at(&self, i: i32) -> i8;

        #[cpp(method = "int use_count() const")]
        pub fn use_count(&self) -> i32;

        pub fn new(sz: i32) -> Self { unique_buffer_new(sz) }
    }
}

hicc::import_class! {
    #[cpp(class = "unique_ptr_ns::Processor")]
    pub class Processor {
        #[cpp(method = "const char* process(const char* input)")]
        pub fn process(&mut self, input: *const i8) -> *const i8;

        pub fn new() -> Self { processor_new() }
    }
}

hicc::import_lib! {
    #![link_name = "unique_ptr"]

    #[cpp(func = "std::unique_ptr<unique_ptr_ns::UniqueBuffer> hicc::make_unique<unique_ptr_ns::UniqueBuffer, int>(int&&)")]
    pub fn unique_buffer_new(sz: i32) -> UniqueBuffer;

    #[cpp(func = "std::unique_ptr<unique_ptr_ns::Processor> hicc::make_unique<unique_ptr_ns::Processor>()")]
    pub fn processor_new() -> Processor;
}
