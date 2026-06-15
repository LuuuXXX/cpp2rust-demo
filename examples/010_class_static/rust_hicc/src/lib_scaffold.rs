// 010_class_static 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「含静态成员的命名空间类」默认生成的 hicc 骨架。
// 默认构造派生 make_unique 工厂（`new`），实例方法 `value`/`increment` 直出；
// 静态方法 `instance_count`/`reset_instance_count` 不在 import_class! 实例方法内，
// 需手写 `lib.rs` 以「全限定自由函数式」绑定，故不在默认支架内。

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
}
