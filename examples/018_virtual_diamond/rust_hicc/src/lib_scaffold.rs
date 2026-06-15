// 018_virtual_diamond 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「菱形虚继承」默认生成的 hicc 骨架。
// 顶点基类 A、虚继承中间类 B/C、汇聚派生类 D 各自独立 `import_class!` 绑定；
// 各类只绑定自身方法（D::compute() 在 C++ 内部访问唯一 A 子对象与 B/C 数据，
// 避免跨虚基类的 this 偏移问题）。int 成员/构造可直出，`lib.rs` 与支架一致。

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
