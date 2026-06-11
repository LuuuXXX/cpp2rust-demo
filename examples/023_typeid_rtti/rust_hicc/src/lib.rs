// AbiClass is required by the `class!` macro expansion below.
use hicc::AbiClass;

hicc::cpp! {
    #include <iostream>
    #include <typeinfo>
    #include <cmath>

    #include "typeid_rtti.h"
}

hicc::import_class! {
    #[cpp(class = "Shape", destroy = "shape_delete")]
    pub class Shape {
        #[cpp(method = "int getType() const")]
        pub fn get_type(&self) -> i32;

        #[cpp(method = "const char* getTypeName() const")]
        pub fn get_type_name(&self) -> *const i8;

        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "typeid_rtti"]

    class Shape;

    #[cpp(func = "Shape* shape_new_circle(double)")]
    pub unsafe fn shape_new_circle(radius: f64) -> Shape;

    #[cpp(func = "Shape* shape_new_rectangle(double, double)")]
    pub unsafe fn shape_new_rectangle(width: f64, height: f64) -> Shape;

    #[cpp(func = "Shape* shape_new_triangle(double, double)")]
    pub unsafe fn shape_new_triangle(base: f64, height: f64) -> Shape;
}
