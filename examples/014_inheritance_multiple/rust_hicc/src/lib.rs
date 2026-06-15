hicc::cpp! {
    #include <iostream>

    #include "inheritance_multiple.h"
}

hicc::import_class! {
    #[cpp(class = "Base1")]
    pub class Base1 {
        #[cpp(method = "int getValue1() const")]
        pub fn get_value1(&self) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Base2")]
    pub class Base2 {
        #[cpp(method = "int getValue2() const")]
        pub fn get_value2(&self) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Derived")]
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

    class Base1;
    class Base2;
    class Derived;

    #[cpp(func = "std::unique_ptr<Base1> std::make_unique<Base1>(int)")]
    pub fn base1_new_with_v(v: i32) -> Base1;

    #[cpp(func = "std::unique_ptr<Base2> std::make_unique<Base2>(int)")]
    pub fn base2_new_with_v(v: i32) -> Base2;

    #[cpp(func = "std::unique_ptr<Derived> std::make_unique<Derived>(int, int, int)")]
    pub fn derived_new_3(v1: i32, v2: i32, dv: i32) -> Derived;
}
