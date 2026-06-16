//! 037_array_basic: std::array 基本操作（命名空间类直接持有容器）。
//!
//! `IntArray` 直接持有固定大小 `std::array<int, 8>`，演示 size/set/get/fill/sum/max/min
//! 等基本操作。hicc 直出无需 extern-C 不透明指针 + `*_delete`，析构由 Rust `Drop` 自动完成。

hicc::cpp! {
    #include "array_basic.h"
}

hicc::import_class! {
    #[cpp(class = "array_basic_ns::IntArray")]
    pub class IntArray {
        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        #[cpp(method = "void set(int i, int v)")]
        pub fn set(&mut self, i: i32, v: i32);

        #[cpp(method = "int get(int i) const")]
        pub fn get(&self, i: i32) -> i32;

        #[cpp(method = "void fill(int v)")]
        pub fn fill(&mut self, v: i32);

        #[cpp(method = "int sum() const")]
        pub fn sum(&self) -> i32;

        #[cpp(method = "int max() const")]
        pub fn max(&self) -> i32;

        #[cpp(method = "int min() const")]
        pub fn min(&self) -> i32;

        pub fn new() -> Self { int_array_new() }
    }
}

hicc::import_lib! {
    #![link_name = "array_basic"]

    #[cpp(func = "std::unique_ptr<array_basic_ns::IntArray> hicc::make_unique<array_basic_ns::IntArray>()")]
    pub fn int_array_new() -> IntArray;
}
