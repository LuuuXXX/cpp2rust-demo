// 014_inheritance_multiple 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「多继承命名空间类」默认生成的 hicc 骨架。
// 两个基类 Base1/Base2 与派生类 Derived 各自独立 `import_class!` 直接绑定真实
// 命名空间类，int 成员/构造均可直出映射；本示例 `lib.rs` 与支架一致（无需手写补全）。

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
