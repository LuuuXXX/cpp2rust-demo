//! 034_vector_basic: std::vector 基本操作（命名空间类直接持有容器）。
//!
//! `IntVector` / `StringVector` 直接持有 `std::vector`，演示 size/capacity/push_back/get/set
//! 等基本操作。hicc 直出无需 extern-C 不透明指针 + `*_delete`，析构由 Rust `Drop` 自动完成。

hicc::cpp! {
    #include "vector_basic.h"
}

hicc::import_class! {
    #[cpp(class = "vector_basic_ns::IntVector")]
    pub class IntVector {
        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        #[cpp(method = "int capacity() const")]
        pub fn capacity(&self) -> i32;

        #[cpp(method = "int empty() const")]
        pub fn empty(&self) -> i32;

        #[cpp(method = "void reserve(int n)")]
        pub fn reserve(&mut self, n: i32);

        #[cpp(method = "void push_back(int v)")]
        pub fn push_back(&mut self, v: i32);

        #[cpp(method = "void pop_back()")]
        pub fn pop_back(&mut self);

        #[cpp(method = "int get(int i) const")]
        pub fn get(&self, i: i32) -> i32;

        #[cpp(method = "void set(int i, int v)")]
        pub fn set(&mut self, i: i32, v: i32);

        #[cpp(method = "int sum() const")]
        pub fn sum(&self) -> i32;

        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        pub fn new() -> Self { int_vector_new() }
    }
}

hicc::import_class! {
    #[cpp(class = "vector_basic_ns::StringVector")]
    pub class StringVector {
        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        #[cpp(method = "void push_back(const char* s)")]
        pub fn push_back(&mut self, s: *const i8);

        #[cpp(method = "const char* get(int i) const")]
        pub fn get(&self, i: i32) -> *const i8;

        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        pub fn new() -> Self { string_vector_new() }
    }
}

hicc::import_lib! {
    #![link_name = "vector_basic"]

    #[cpp(func = "std::unique_ptr<vector_basic_ns::IntVector> hicc::make_unique<vector_basic_ns::IntVector>()")]
    pub fn int_vector_new() -> IntVector;

    #[cpp(func = "std::unique_ptr<vector_basic_ns::StringVector> hicc::make_unique<vector_basic_ns::StringVector>()")]
    pub fn string_vector_new() -> StringVector;
}
