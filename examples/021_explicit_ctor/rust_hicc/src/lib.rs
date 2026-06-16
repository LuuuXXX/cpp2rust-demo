//! 021_explicit_ctor: 显式构造函数（命名空间类 + 多构造 make_unique 工厂）。
//!
//! Widget 有两个公有构造：`Widget(int)`（非 explicit）与 `explicit Widget(double)`。
//! hicc 直出为每个公有构造各派生一条 `make_unique` 工厂与关联函数：`new(i32)` 与
//! `new_2(f64)`。C++ 的 explicit 关键字只约束「隐式转换」，在 Rust 侧两者都是显式的
//! 关联函数调用，故 explicit 与否不影响直出绑定。

hicc::cpp! {
    #include "explicit_ctor.h"
}

hicc::import_class! {
    #[cpp(class = "explicit_ctor_ns::Widget")]
    pub class Widget {
        #[cpp(method = "int getValue() const")]
        pub fn get_value(&self) -> i32;

        pub fn new(v: i32) -> Self { widget_new(v) }
        pub fn new_2(v: f64) -> Self { widget_new_2(v) }
    }
}

hicc::import_lib! {
    #![link_name = "explicit_ctor"]

    #[cpp(func = "std::unique_ptr<explicit_ctor_ns::Widget> hicc::make_unique<explicit_ctor_ns::Widget, int>(int&&)")]
    pub fn widget_new(v: i32) -> Widget;

    #[cpp(func = "std::unique_ptr<explicit_ctor_ns::Widget> hicc::make_unique<explicit_ctor_ns::Widget, double>(double&&)")]
    pub fn widget_new_2(v: f64) -> Widget;
}
