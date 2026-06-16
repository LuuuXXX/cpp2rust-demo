//! 022_mutable_member: mutable 成员（命名空间类 + make_unique 工厂）。
//!
//! `DataFetcher` 的 `access_count_` 被 `mutable` 修饰，故 const 方法 `fetch() const`
//! 也能修改它。hicc 直出把 const 方法映射为 `&self`：可变更新发生在 C++ 侧（受
//! `mutable` 约束），Rust 侧以共享引用调用即可——这正是 mutable 提供的「逻辑常量、
//! 物理可变」内部可变性。

hicc::cpp! {
    #include "mutable_member.h"
}

hicc::import_class! {
    #[cpp(class = "mutable_member_ns::DataFetcher")]
    pub class DataFetcher {
        #[cpp(method = "int fetch() const")]
        pub fn fetch(&self) -> i32;

        #[cpp(method = "int accessCount() const")]
        pub fn access_count(&self) -> i32;

        pub fn new(seed: i32) -> Self { data_fetcher_new(seed) }
    }
}

hicc::import_lib! {
    #![link_name = "mutable_member"]

    #[cpp(func = "std::unique_ptr<mutable_member_ns::DataFetcher> hicc::make_unique<mutable_member_ns::DataFetcher, int>(int&&)")]
    pub fn data_fetcher_new(seed: i32) -> DataFetcher;
}
