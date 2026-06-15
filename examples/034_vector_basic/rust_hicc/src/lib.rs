hicc::cpp! {
    #include <iostream>
    #include <vector>
    #include <string>
    #include <cstring>

    #include "vector_basic.h"
}

hicc::import_class! {
    #[cpp(class = "IntVector")]
    pub class IntVector {
        #[cpp(method = "void push_back(int val)")]
        pub fn push_back(&mut self, val: i32);

        #[cpp(method = "int get(size_t i) const")]
        pub fn get(&self, i: usize) -> i32;

        #[cpp(method = "void set(size_t i, int val)")]
        pub fn set(&mut self, i: usize, val: i32);

        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;

        #[cpp(method = "bool empty() const")]
        pub fn empty(&self) -> bool;

        #[cpp(method = "size_t capacity() const")]
        pub fn capacity(&self) -> usize;

        #[cpp(method = "void reserve(size_t n)")]
        pub fn reserve(&mut self, n: usize);

        #[cpp(method = "int* data()")]
        pub fn data(&mut self) -> *mut i32;

        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);
    }
}

hicc::import_class! {
    #[cpp(class = "StringVector")]
    pub class StringVector {
        #[cpp(method = "size_t size() const")]
        pub fn size(&self) -> usize;
    }
}

hicc::import_lib! {
    #![link_name = "vector_basic"]

    class IntVector;
    class StringVector;

    #[cpp(func = "std::unique_ptr<IntVector> hicc::make_unique<IntVector>()")]
    pub fn int_vector_new() -> IntVector;

    #[cpp(func = "std::unique_ptr<StringVector> hicc::make_unique<StringVector>()")]
    pub fn string_vector_new() -> StringVector;
}
