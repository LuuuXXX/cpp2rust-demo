hicc::cpp! {
    #include <iostream>

    #include "virtual_diamond.h"
    std::unique_ptr<A> a_new(int v) { return std::make_unique<A>(v); }
    std::unique_ptr<B> b_new(int a, int b) { return std::make_unique<B>(a, b); }
    std::unique_ptr<C> c_new(int a, int c) { return std::make_unique<C>(a, c); }
    std::unique_ptr<D> d_new(int a, int b, int c, int d) { return std::make_unique<D>(a, b, c, d); }
    int d_get_a_value(D* self) { return self->getAValue(); }
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
        #[cpp(method = "int getBValue() const")]
        pub fn get_b_value(&self) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "C")]
    pub class C {
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

    #[cpp(func = "std::unique_ptr<A> a_new(int)")]
    pub fn a_new_with_v(v: i32) -> A;

    #[cpp(func = "std::unique_ptr<B> b_new(int, int)")]
    pub fn b_new_2(a: i32, b: i32) -> B;

    #[cpp(func = "std::unique_ptr<C> c_new(int, int)")]
    pub fn c_new_2(a: i32, c: i32) -> C;

    #[cpp(func = "std::unique_ptr<D> d_new(int, int, int, int)")]
    pub fn d_new_4(a: i32, b: i32, c: i32, d: i32) -> D;

    #[cpp(func = "int d_get_a_value(D*)")]
    pub fn d_get_a_value(self_: *mut D) -> i32;
}
