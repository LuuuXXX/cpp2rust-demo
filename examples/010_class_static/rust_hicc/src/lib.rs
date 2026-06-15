//! 010_class_static: 静态成员（命名空间类 + make_unique 工厂 + 静态方法绑定）。
//!
//! hicc 模式：默认构造派生 make_unique 工厂，实例方法直出；静态方法以
//! 「全限定自由函数式」绑定（`Counter::instance_count()`），由手写补全
//! （工具默认支架不含静态方法，见 `lib_scaffold.rs`）。析构由 hicc `Drop` 负责，
//! 静态计数随实例 Drop 自动维护。

hicc::cpp! {
    #include "class_static.h"
}

hicc::import_class! {
    #[cpp(class = "class_static_ns::Counter")]
    pub class Counter {
        #[cpp(method = "int value() const")]
        pub fn value(&self) -> i32;

        #[cpp(method = "void increment()")]
        pub fn increment(&mut self);

        pub fn new() -> Self { counter_new() }
    }
}

hicc::import_lib! {
    #![link_name = "class_static"]

    #[cpp(func = "std::unique_ptr<class_static_ns::Counter> hicc::make_unique<class_static_ns::Counter>()")]
    pub fn counter_new() -> Counter;

    // 静态方法：全限定自由函数式调用。
    #[cpp(func = "int class_static_ns::Counter::instance_count()")]
    pub fn counter_instance_count() -> i32;

    #[cpp(func = "void class_static_ns::Counter::reset_instance_count()")]
    pub fn counter_reset_instance_count();
}
