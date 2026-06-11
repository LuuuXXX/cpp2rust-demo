hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <string>

    #include "virtual_override.h"
}

hicc::import_class! {
    #[cpp(class = "Base", destroy = "base_delete")]
    pub class Base {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Derived", destroy = "derived_delete")]
    pub class Derived {
        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;

        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "double getValue() const")]
        pub fn get_value(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "virtual_override"]

    class Base;
    class Derived;

    #[cpp(func = "Base* base_create(int)")]
    pub unsafe fn base_create(type_: i32) -> Base;

    #[cpp(func = "Derived* derived_new(double)")]
    pub fn derived_new(value: f64) -> Derived;
}
