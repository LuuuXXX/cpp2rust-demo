// 017_virtual_override 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「显式 override 覆写虚函数」默认生成的 hicc 骨架。
// 基类 Base 的虚函数 area() 与派生类 Derived 的 override 覆写各自绑定；double 成员/构造
// 均可直出映射，本示例 `lib.rs` 与支架一致（无需手写补全）。

hicc::cpp! {
    #include "virtual_override.h"
}

hicc::import_class! {
    #[cpp(class = "virtual_override_ns::Base")]
    pub class Base {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        pub fn new() -> Self { base_new() }
    }
}

hicc::import_class! {
    #[cpp(class = "virtual_override_ns::Derived")]
    pub class Derived {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "double value() const")]
        pub fn value(&self) -> f64;

        pub fn new(v: f64) -> Self { derived_new(v) }
    }
}

hicc::import_lib! {
    #![link_name = "virtual_override"]

    #[cpp(func = "std::unique_ptr<virtual_override_ns::Base> hicc::make_unique<virtual_override_ns::Base>()")]
    pub fn base_new() -> Base;

    #[cpp(func = "std::unique_ptr<virtual_override_ns::Derived> hicc::make_unique<virtual_override_ns::Derived, double>(double&&)")]
    pub fn derived_new(v: f64) -> Derived;
}
