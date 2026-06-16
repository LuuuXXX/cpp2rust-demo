//! 035_map_basic: std::map / std::unordered_map 基本操作（命名空间类直接持有容器）。
//!
//! `StringIntMap` / `Counter` 直接持有 STL 关联容器，演示 insert/get/contains/erase
//! 与词频计数等基本操作。hicc 直出无需 extern-C 不透明指针 + `*_delete`，析构由 Rust `Drop` 自动完成。

hicc::cpp! {
    #include "map_basic.h"
}

hicc::import_class! {
    #[cpp(class = "map_basic_ns::StringIntMap")]
    pub class StringIntMap {
        #[cpp(method = "void insert(const char* key, int value)")]
        pub fn insert(&mut self, key: *const i8, value: i32);

        #[cpp(method = "int get(const char* key) const")]
        pub fn get(&self, key: *const i8) -> i32;

        #[cpp(method = "int contains(const char* key) const")]
        pub fn contains(&self, key: *const i8) -> i32;

        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        #[cpp(method = "int erase(const char* key)")]
        pub fn erase(&mut self, key: *const i8) -> i32;

        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        #[cpp(method = "const char* first_key() const")]
        pub fn first_key(&self) -> *const i8;

        pub fn new() -> Self { string_int_map_new() }
    }
}

hicc::import_class! {
    #[cpp(class = "map_basic_ns::Counter")]
    pub class Counter {
        #[cpp(method = "void add(const char* word)")]
        pub fn add(&mut self, word: *const i8);

        #[cpp(method = "int count(const char* word) const")]
        pub fn count(&self, word: *const i8) -> i32;

        #[cpp(method = "int unique_words() const")]
        pub fn unique_words(&self) -> i32;

        #[cpp(method = "const char* last_word() const")]
        pub fn last_word(&self) -> *const i8;

        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        pub fn new() -> Self { counter_new() }
    }
}

hicc::import_lib! {
    #![link_name = "map_basic"]

    #[cpp(func = "std::unique_ptr<map_basic_ns::StringIntMap> hicc::make_unique<map_basic_ns::StringIntMap>()")]
    pub fn string_int_map_new() -> StringIntMap;

    #[cpp(func = "std::unique_ptr<map_basic_ns::Counter> hicc::make_unique<map_basic_ns::Counter>()")]
    pub fn counter_new() -> Counter;
}
