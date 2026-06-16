//! 032_placement_new: 定位 new（命名空间类在预分配存储中构造对象）。
//!
//! `Buffer` 在指定偏移处用 placement new 构造 `SimpleValue`，`ObjectArray` 以元素槽位
//! 逐个构造，模拟 `std::vector` 底层内存管理。hicc 直出无需手写 `*_delete`，存储由
//! Rust `Drop` 自动回收。

hicc::cpp! {
    #include "placement_new.h"
}

hicc::import_class! {
    #[cpp(class = "placement_new_ns::Buffer")]
    pub class Buffer {
        #[cpp(method = "int capacity() const")]
        pub fn capacity(&self) -> i32;

        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        #[cpp(method = "int construct_at(int offset, int v)")]
        pub fn construct_at(&mut self, offset: i32, v: i32) -> i32;

        #[cpp(method = "int value_at(int offset) const")]
        pub fn value_at(&self, offset: i32) -> i32;

        pub fn new(capacity: i32) -> Self { buffer_new(capacity) }
    }
}

hicc::import_class! {
    #[cpp(class = "placement_new_ns::ObjectArray")]
    pub class ObjectArray {
        #[cpp(method = "int count() const")]
        pub fn count(&self) -> i32;

        #[cpp(method = "int element_size() const")]
        pub fn element_size(&self) -> i32;

        #[cpp(method = "int emplace(int i, int v)")]
        pub fn emplace(&mut self, i: i32, v: i32) -> i32;

        #[cpp(method = "int at(int i) const")]
        pub fn at(&self, i: i32) -> i32;

        pub fn new(count: i32) -> Self { object_array_new(count) }
    }
}

hicc::import_lib! {
    #![link_name = "placement_new"]

    #[cpp(func = "std::unique_ptr<placement_new_ns::Buffer> hicc::make_unique<placement_new_ns::Buffer, int>(int&&)")]
    pub fn buffer_new(capacity: i32) -> Buffer;

    #[cpp(func = "std::unique_ptr<placement_new_ns::ObjectArray> hicc::make_unique<placement_new_ns::ObjectArray, int>(int&&)")]
    pub fn object_array_new(count: i32) -> ObjectArray;
}
