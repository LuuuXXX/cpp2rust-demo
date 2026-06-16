//! 038_tuple_basic: std::tuple 基本操作（命名空间类直接持有 tuple）。
//!
//! `Record` 直接持有 `std::tuple<int, double, std::string>`，演示 id/score/name
//! 等基本操作。hicc 直出无需 extern-C 不透明指针 + `*_delete`，析构由 Rust `Drop` 自动完成。

hicc::cpp! {
    #include "tuple_basic.h"
}

hicc::import_class! {
    #[cpp(class = "tuple_basic_ns::Record")]
    pub class Record {
        #[cpp(method = "int id() const")]
        pub fn id(&self) -> i32;

        #[cpp(method = "double score() const")]
        pub fn score(&self) -> f64;

        #[cpp(method = "const char* name() const")]
        pub fn name(&self) -> *const i8;

        #[cpp(method = "void set_id(int id)")]
        pub fn set_id(&mut self, id: i32);

        #[cpp(method = "void set_score(double score)")]
        pub fn set_score(&mut self, score: f64);

        pub fn new(id: i32, score: f64, name: *const i8) -> Self { record_new(id, score, name) }
    }
}

hicc::import_lib! {
    #![link_name = "tuple_basic"]

    #[cpp(func = "std::unique_ptr<tuple_basic_ns::Record> hicc::make_unique<tuple_basic_ns::Record, int, double, const char*>(int&&, double&&, const char*&&)")]
    pub fn record_new(id: i32, score: f64, name: *const i8) -> Record;
}
