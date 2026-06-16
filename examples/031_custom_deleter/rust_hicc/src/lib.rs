//! 031_custom_deleter: 自定义删除器（命名空间类内部用 unique_ptr<T, Deleter>）。
//!
//! `ManagedResource` 用带自定义删除器的 `std::unique_ptr` 持有负载，演示 RAII 自定义
//! 删除策略。hicc 直出无需手写 `*_delete`，对象析构由 Rust `Drop` 自动完成，届时内部
//! `unique_ptr` 会调用自定义删除器（`cleanup_count` 可观测其被触发）。

hicc::cpp! {
    #include "custom_deleter.h"
}

hicc::import_class! {
    #[cpp(class = "custom_deleter_ns::ManagedResource")]
    pub class ManagedResource {
        #[cpp(method = "const char* name() const")]
        pub fn name(&self) -> *const i8;

        #[cpp(method = "int released() const")]
        pub fn released(&self) -> i32;

        #[cpp(method = "void release()")]
        pub fn release(&mut self);

        pub fn new(name: *const i8) -> Self { managed_resource_new(name) }
    }
}

hicc::import_lib! {
    #![link_name = "custom_deleter"]

    #[cpp(func = "std::unique_ptr<custom_deleter_ns::ManagedResource> hicc::make_unique<custom_deleter_ns::ManagedResource, const char*>(const char*&&)")]
    pub fn managed_resource_new(name: *const i8) -> ManagedResource;

    #[cpp(func = "int custom_deleter_ns::cleanup_count()")]
    pub fn cleanup_count() -> i32;
}
