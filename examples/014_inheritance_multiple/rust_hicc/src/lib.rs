//! 014_inheritance_multiple: 多继承（命名空间类 + make_unique 工厂）。
//!
//! hicc 直出：两个基类 `Base1`/`Base2` 与派生类
//! `Derived : public Base1, public Base2` 各自以 `import_class!` 直接绑定真实
//! 命名空间类。所有成员/构造均为 `int`，可直出映射，故本示例 `lib.rs` 与工具默认
//! 支架（`lib_scaffold.rs`）一致（无需手写补全）。
//!
//! 派生类 `compute()` 复用两个基类的数据成员（`value1_ + value2_ + derived_value_`），
//! 体现多继承的数据组合；按 hicc 约束，派生类绑定块只声明自身方法，不重复绑定继承
//! 而来的 `value1()`/`value2()`（多继承下基类 `this` 偏移不同）。

hicc::cpp! {
    #include "inheritance_multiple.h"
}

hicc::import_class! {
    #[cpp(class = "inheritance_multiple_ns::Base1")]
    pub class Base1 {
        #[cpp(method = "int value1() const")]
        pub fn value1(&self) -> i32;

        pub fn new(v: i32) -> Self { base1_new(v) }
    }
}

hicc::import_class! {
    #[cpp(class = "inheritance_multiple_ns::Base2")]
    pub class Base2 {
        #[cpp(method = "int value2() const")]
        pub fn value2(&self) -> i32;

        pub fn new(v: i32) -> Self { base2_new(v) }
    }
}

hicc::import_class! {
    #[cpp(class = "inheritance_multiple_ns::Derived")]
    pub class Derived {
        #[cpp(method = "int derived_value() const")]
        pub fn derived_value(&self) -> i32;

        #[cpp(method = "int compute() const")]
        pub fn compute(&self) -> i32;

        pub fn new(v1: i32, v2: i32, dv: i32) -> Self { derived_new(v1, v2, dv) }
    }
}

hicc::import_lib! {
    #![link_name = "inheritance_multiple"]

    #[cpp(func = "std::unique_ptr<inheritance_multiple_ns::Base1> hicc::make_unique<inheritance_multiple_ns::Base1, int>(int&&)")]
    pub fn base1_new(v: i32) -> Base1;

    #[cpp(func = "std::unique_ptr<inheritance_multiple_ns::Base2> hicc::make_unique<inheritance_multiple_ns::Base2, int>(int&&)")]
    pub fn base2_new(v: i32) -> Base2;

    #[cpp(func = "std::unique_ptr<inheritance_multiple_ns::Derived> hicc::make_unique<inheritance_multiple_ns::Derived, int, int, int>(int&&, int&&, int&&)")]
    pub fn derived_new(v1: i32, v2: i32, dv: i32) -> Derived;
}
