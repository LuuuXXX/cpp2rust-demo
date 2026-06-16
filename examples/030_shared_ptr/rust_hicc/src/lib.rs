//! 030_shared_ptr: 共享所有权（命名空间类内部使用 std::shared_ptr）。
//!
//! `SharedData` 内部以 `std::shared_ptr` 持有负载，`Cache` 用 `vector<shared_ptr>` 缓存，
//! 演示引用计数与共享语义。hicc 直出无需 extern-C shim，析构由 Rust `Drop` 自动完成，
//! 相当于 Rust 的 `Arc<T>`。

hicc::cpp! {
    #include "shared_ptr.h"
}

hicc::import_class! {
    #[cpp(class = "shared_ptr_ns::SharedData")]
    pub class SharedData {
        #[cpp(method = "const char* name() const")]
        pub fn name(&self) -> *const i8;

        #[cpp(method = "int use_count() const")]
        pub fn use_count(&self) -> i32;

        #[cpp(method = "void reset()")]
        pub fn reset(&mut self);

        #[cpp(method = "int expired() const")]
        pub fn expired(&self) -> i32;

        pub fn new(name: *const i8) -> Self { shared_data_new(name) }
    }
}

hicc::import_class! {
    #[cpp(class = "shared_ptr_ns::Cache")]
    pub class Cache {
        #[cpp(method = "int store(const char* name)")]
        pub fn store(&mut self, name: *const i8) -> i32;

        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        #[cpp(method = "void clear()")]
        pub fn clear(&mut self);

        pub fn new() -> Self { cache_new() }
    }
}

hicc::import_lib! {
    #![link_name = "shared_ptr"]

    #[cpp(func = "std::unique_ptr<shared_ptr_ns::SharedData> hicc::make_unique<shared_ptr_ns::SharedData, const char*>(const char*&&)")]
    pub fn shared_data_new(name: *const i8) -> SharedData;

    #[cpp(func = "std::unique_ptr<shared_ptr_ns::Cache> hicc::make_unique<shared_ptr_ns::Cache>()")]
    pub fn cache_new() -> Cache;
}
