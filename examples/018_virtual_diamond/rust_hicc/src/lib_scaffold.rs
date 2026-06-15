// 此文件为 cpp2rust-demo 工具对 018_virtual_diamond 自动生成的支架黄金文件，
// 仅供 L1 golden 测试（test_018_virtual_diamond）进行生成准确性验证。
//
// Direct 模式下，工具为所有 4 个类（A/B/C/D）生成 import_class! + make_unique 工厂。
// d_get_a_value(D* self) 是工具生成的 C++ 包装函数（虚继承中 getAValue 的
// this 调整量问题需要辅助函数），同时作为 import_lib! 中的 StaticAccessor 绑定。
hicc::cpp! {
    #include <iostream>

    #include "virtual_diamond.h"
    int d_get_a_value(D* self) {
    return self->getAValue();
    }
}

hicc::import_class! {
    #[cpp(class = "A")]
    pub class A {
        #[cpp(method = "int getAValue() const")]
        pub fn get_a_value(&self) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "B")]
    pub class B {
        #[cpp(method = "int getAValue() const")]
        pub fn get_a_value(&self) -> i32;

        #[cpp(method = "int getBValue() const")]
        pub fn get_b_value(&self) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "C")]
    pub class C {
        #[cpp(method = "int getAValue() const")]
        pub fn get_a_value(&self) -> i32;

        #[cpp(method = "int getCValue() const")]
        pub fn get_c_value(&self) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "D")]
    pub class D {
        #[cpp(method = "int getBValue() const")]
        pub fn get_b_value(&self) -> i32;

        #[cpp(method = "int getCValue() const")]
        pub fn get_c_value(&self) -> i32;

        #[cpp(method = "int getDValue() const")]
        pub fn get_d_value(&self) -> i32;

        #[cpp(method = "void compute() const")]
        pub fn compute(&self);
    }
}

hicc::import_lib! {
    #![link_name = "virtual_diamond"]

    class A;
    class B;
    class C;
    class D;

    #[cpp(func = "std::unique_ptr<A> std::make_unique<A>(int)")]
    pub fn a_new_with_v(v: i32) -> A;

    #[cpp(func = "std::unique_ptr<B> std::make_unique<B>(int, int)")]
    pub fn b_new_2(a: i32, b: i32) -> B;

    #[cpp(func = "std::unique_ptr<C> std::make_unique<C>(int, int)")]
    pub fn c_new_2(a: i32, c: i32) -> C;

    #[cpp(func = "std::unique_ptr<D> std::make_unique<D>(int, int, int, int)")]
    pub fn d_new_4(a: i32, b: i32, c: i32, d: i32) -> D;

    #[cpp(func = "int d_get_a_value(D*)")]
    pub fn d_get_a_value(self_: *mut D) -> i32;
}
