// 006_class_basic 工具默认产物支架（hicc 直出，去 shim）。
//
// 本文件用于 L1 黄金比对：校验 `cpp2rust-demo init` 对 idiomatic 命名空间类
// 默认生成的 hicc 三段式骨架。手写 `lib.rs` 在此基础上额外提供 `name()`
// （`const std::string&` → `ClassRef<string>`）与 `with_name`（`std::string` 构造）
// 等 hicc 直出无法默认推导、需结合 `hicc_std` 类型补全的绑定。

hicc::cpp! {
    #include "class_basic.h"
}

hicc::import_class! {
    #[cpp(class = "class_basic_ns::Counter")]
    pub class Counter {
        #[cpp(method = "void inc()")]
        pub fn inc(&mut self);

        #[cpp(method = "void inc_by(int delta)")]
        pub fn inc_by(&mut self, delta: i32);

        #[cpp(method = "void reset()")]
        pub fn reset(&mut self);

        #[cpp(method = "int count() const")]
        pub fn count(&self) -> i32;

        pub fn new() -> Self { counter_new() }
    }
}

hicc::import_lib! {
    #![link_name = "class_basic"]

    #[cpp(func = "std::unique_ptr<class_basic_ns::Counter> hicc::make_unique<class_basic_ns::Counter>()")]
    pub fn counter_new() -> Counter;
}
