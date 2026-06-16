//! 017_virtual_override: 显式 override 覆写虚函数（命名空间类 + make_unique 工厂）。
//!
//! hicc 直出：基类 `Base` 声明虚函数 `area()`（默认 0），派生类
//! `Derived : public Base` 用 `override` 关键字显式覆写 `area()`（`value_ * value_`）。
//! 两类各自以 `import_class!` 直接绑定真实命名空间类；所有成员/构造均为 `double`/无参，
//! 可直出映射，故本示例 `lib.rs` 与工具默认支架（`lib_scaffold.rs`）一致。

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
