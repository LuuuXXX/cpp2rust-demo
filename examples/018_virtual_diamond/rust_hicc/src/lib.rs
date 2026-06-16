//! 018_virtual_diamond: 菱形虚继承（命名空间类 + make_unique 工厂）。
//!
//! hicc 直出：`A` 为顶点基类，`B`/`C` 各以 `virtual public A` 虚继承，`D : public B, C`
//! 汇聚——虚继承保证 `A` 子对象唯一。四个类各自以 `import_class!` 直接绑定真实命名空间类。
//!
//! 关键约束：各派生类**只绑定自身方法**，不在派生类绑定块重复声明继承而来的方法。
//! 跨虚基类调用继承方法（如在 `D` 块声明 `A::a_value()`）会因 hicc 的 `this` 偏移截断
//! 而出错；本示例改由 `D::compute()`（C++ 内部访问唯一 `A` 子对象及 `B`/`C` 数据，返回
//! 四者之和）体现菱形数据汇聚，规避该问题。所有成员/构造均为 `int`，`lib.rs` 与支架一致。

hicc::cpp! {
    #include "virtual_diamond.h"
}

hicc::import_class! {
    #[cpp(class = "virtual_diamond_ns::A")]
    pub class A {
        #[cpp(method = "int a_value() const")]
        pub fn a_value(&self) -> i32;

        pub fn new(v: i32) -> Self { a_new(v) }
    }
}

hicc::import_class! {
    #[cpp(class = "virtual_diamond_ns::B")]
    pub class B {
        #[cpp(method = "int b_value() const")]
        pub fn b_value(&self) -> i32;

        pub fn new(a: i32, b: i32) -> Self { b_new(a, b) }
    }
}

hicc::import_class! {
    #[cpp(class = "virtual_diamond_ns::C")]
    pub class C {
        #[cpp(method = "int c_value() const")]
        pub fn c_value(&self) -> i32;

        pub fn new(a: i32, c: i32) -> Self { c_new(a, c) }
    }
}

hicc::import_class! {
    #[cpp(class = "virtual_diamond_ns::D")]
    pub class D {
        #[cpp(method = "int d_value() const")]
        pub fn d_value(&self) -> i32;

        #[cpp(method = "int compute() const")]
        pub fn compute(&self) -> i32;

        pub fn new(a: i32, b: i32, c: i32, d: i32) -> Self { d_new(a, b, c, d) }
    }
}

hicc::import_lib! {
    #![link_name = "virtual_diamond"]

    #[cpp(func = "std::unique_ptr<virtual_diamond_ns::A> hicc::make_unique<virtual_diamond_ns::A, int>(int&&)")]
    pub fn a_new(v: i32) -> A;

    #[cpp(func = "std::unique_ptr<virtual_diamond_ns::B> hicc::make_unique<virtual_diamond_ns::B, int, int>(int&&, int&&)")]
    pub fn b_new(a: i32, b: i32) -> B;

    #[cpp(func = "std::unique_ptr<virtual_diamond_ns::C> hicc::make_unique<virtual_diamond_ns::C, int, int>(int&&, int&&)")]
    pub fn c_new(a: i32, c: i32) -> C;

    #[cpp(func = "std::unique_ptr<virtual_diamond_ns::D> hicc::make_unique<virtual_diamond_ns::D, int, int, int, int>(int&&, int&&, int&&, int&&)")]
    pub fn d_new(a: i32, b: i32, c: i32, d: i32) -> D;
}
