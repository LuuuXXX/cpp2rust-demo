hicc::cpp! {
    #include <iostream>

    #include "inheritance_multiple.h"
}

hicc::import_class! {
    #[cpp(class = "Derived", destroy = "derived_delete")]
    pub class Derived {
        #[cpp(method = "int getValue1() const")]
        pub fn get_value1(&self) -> i32;

        #[cpp(method = "int getValue2() const")]
        pub fn get_value2(&self) -> i32;

        #[cpp(method = "int getDerivedValue() const")]
        pub fn get_derived_value(&self) -> i32;

        #[cpp(method = "void compute() const")]
        pub fn compute(&self);
    }
}

hicc::import_lib! {
    #![link_name = "inheritance_multiple"]

    class Derived;

    #[cpp(func = "Derived* derived_new(int, int, int)")]
    pub fn derived_new(v1: i32, v2: i32, dv: i32) -> Derived;
}
