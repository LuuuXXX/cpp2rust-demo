// 007_class_constructor 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「多构造函数命名空间类」默认生成的 hicc 骨架。
// 默认构造与 `int` 构造派生 make_unique 工厂（`new`/`new_2`）；含 `std::string`
// 参数的构造与返回 `const std::string&` 的 `name()` 方法需手写 `lib.rs` 用
// `hicc_std::string` 补全，故不在默认支架内。

hicc::cpp! {
    #include "class_constructor.h"
}

hicc::import_class! {
    #[cpp(class = "class_ctor_ns::Widget")]
    pub class Widget {
        #[cpp(method = "int value() const")]
        pub fn value(&self) -> i32;

        pub fn new() -> Self { widget_new() }

        pub fn new_2(v: i32) -> Self { widget_new_2(v) }
    }
}

hicc::import_lib! {
    #![link_name = "class_constructor"]

    #[cpp(func = "std::unique_ptr<class_ctor_ns::Widget> hicc::make_unique<class_ctor_ns::Widget>()")]
    pub fn widget_new() -> Widget;

    #[cpp(func = "std::unique_ptr<class_ctor_ns::Widget> hicc::make_unique<class_ctor_ns::Widget, int>(int&&)")]
    pub fn widget_new_2(v: i32) -> Widget;
}
