//! 008_class_copy: 深拷贝构造（命名空间类 + make_unique 工厂）。
//!
//! hicc 模式：默认 / `int` 构造与拷贝构造各派生一个 make_unique 工厂，
//! Rust 侧以关联函数包装。拷贝构造 `Buffer(const Buffer&)` 由手写补全
//! （工具默认支架排除拷贝/移动构造，见 `lib_scaffold.rs`）。

hicc::cpp! {
    #include "class_copy.h"
}

hicc::import_class! {
    #[cpp(class = "class_copy_ns::Buffer")]
    pub class Buffer {
        #[cpp(method = "void set(int index, int value)")]
        pub fn set(&mut self, index: i32, value: i32);

        #[cpp(method = "int get(int index) const")]
        pub fn get(&self, index: i32) -> i32;

        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        pub fn new() -> Self { buffer_new() }

        pub fn new_2(sz: i32) -> Self { buffer_new_2(sz) }

        // 深拷贝：借用 other 构造独立副本。
        pub fn from_copy(other: &Buffer) -> Self { buffer_from_copy(other) }
    }
}

hicc::import_lib! {
    #![link_name = "class_copy"]

    #[cpp(func = "std::unique_ptr<class_copy_ns::Buffer> hicc::make_unique<class_copy_ns::Buffer>()")]
    pub fn buffer_new() -> Buffer;

    #[cpp(func = "std::unique_ptr<class_copy_ns::Buffer> hicc::make_unique<class_copy_ns::Buffer, int>(int&&)")]
    pub fn buffer_new_2(sz: i32) -> Buffer;

    #[cpp(func = "std::unique_ptr<class_copy_ns::Buffer> hicc::make_unique<class_copy_ns::Buffer, const class_copy_ns::Buffer&>(const class_copy_ns::Buffer&)")]
    pub fn buffer_from_copy(other: &Buffer) -> Buffer;
}
