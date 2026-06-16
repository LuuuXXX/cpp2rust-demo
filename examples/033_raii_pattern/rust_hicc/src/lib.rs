//! 033_raii_pattern: RAII 资源管理（命名空间类，构造获取/析构释放）。
//!
//! `Resource` 构造时获取资源（活跃计数 +1）、析构时释放（计数 -1）；`Transaction` 是作用域
//! 守卫，未 `commit()` 即析构则自动回滚。hicc 直出无需手写 `*_delete`，析构由 Rust `Drop`
//! 自动触发，RAII 语义与 C++ 一致。

hicc::cpp! {
    #include "raii_pattern.h"
}

hicc::import_class! {
    #[cpp(class = "raii_pattern_ns::Resource")]
    pub class Resource {
        #[cpp(method = "const char* name() const")]
        pub fn name(&self) -> *const i8;

        pub fn new(name: *const i8) -> Self { resource_new(name) }
    }
}

hicc::import_class! {
    #[cpp(class = "raii_pattern_ns::Transaction")]
    pub class Transaction {
        #[cpp(method = "void commit()")]
        pub fn commit(&mut self);

        #[cpp(method = "int committed() const")]
        pub fn committed(&self) -> i32;

        pub fn new() -> Self { transaction_new() }
    }
}

hicc::import_lib! {
    #![link_name = "raii_pattern"]

    #[cpp(func = "std::unique_ptr<raii_pattern_ns::Resource> hicc::make_unique<raii_pattern_ns::Resource, const char*>(const char*&&)")]
    pub fn resource_new(name: *const i8) -> Resource;

    #[cpp(func = "std::unique_ptr<raii_pattern_ns::Transaction> hicc::make_unique<raii_pattern_ns::Transaction>()")]
    pub fn transaction_new() -> Transaction;

    #[cpp(func = "int raii_pattern_ns::active_count()")]
    pub fn active_count() -> i32;

    #[cpp(func = "int raii_pattern_ns::rollback_count()")]
    pub fn rollback_count() -> i32;
}
