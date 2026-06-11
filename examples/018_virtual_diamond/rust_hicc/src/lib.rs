hicc::cpp! {
    #include <iostream>

    #include "virtual_diamond.h"
    int d_get_a_value(D* self) {
        return self->getAValue();
    }

}

hicc::import_class! {
    #[cpp(class = "D", destroy = "d_delete")]
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

    class D;

    #[cpp(func = "D* d_new(int, int, int, int)")]
    pub unsafe fn d_new(a: i32, b: i32, c: i32, d: i32) -> D;

    #[cpp(func = "int d_get_a_value(D*)")]
    pub fn d_get_a_value(self_: *mut D) -> i32;
}
