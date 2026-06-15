//! 007_class_constructor: 多构造函数 + 析构。
//!
//! hicc 模式：每个构造函数一个工厂 + Rust 关联函数包装

hicc::cpp! {
    #include "class_constructor.h"
    #include <hicc/std/string.hpp>
}

hicc::import_class! {
    class string = hicc_std::string;

    #[cpp(class = "class_ctor_ns::Widget")]
    pub class Widget {
        #[cpp(method = "const std::string& name() const")]
        pub fn name(&self) -> &string;

        #[cpp(method = "int value() const")]
        pub fn value(&self) -> i32;

        pub fn new() -> Self { widget_default() }
        pub fn from_int(v: i32) -> Self { widget_from_int(v) }
        pub fn from_named(name: string, v: i32) -> Self { widget_from_named(name, v) }
    }
}

hicc::import_lib! {
    #![link_name = "class_constructor"]

    #[cpp(func = "std::unique_ptr<class_ctor_ns::Widget> hicc::make_unique<class_ctor_ns::Widget>()")]
    pub fn widget_default() -> Widget;

    #[cpp(func = "std::unique_ptr<class_ctor_ns::Widget> hicc::make_unique<class_ctor_ns::Widget, int>(int&&)")]
    pub fn widget_from_int(v: i32) -> Widget;

    #[cpp(func = "std::unique_ptr<class_ctor_ns::Widget> hicc::make_unique<class_ctor_ns::Widget, std::string, int>(std::string&&, int&&)")]
    pub fn widget_from_named(name: hicc_std::string, v: i32) -> Widget;
}
