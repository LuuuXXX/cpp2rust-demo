// AbiClass is required by the `class!` macro expansion below.
use hicc::AbiClass;

hicc::cpp! {
    #include <iostream>
    #include <cmath>
    #include <cstring>

    #include "virtual_pure.h"
}

hicc::import_class! {
    #[cpp(class = "AbstractShape", destroy = "abstract_shape_delete")]
    pub class AbstractShape {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "virtual_pure"]

    class AbstractShape;

    #[cpp(func = "AbstractShape* abstract_shape_create_circle(double)")]
    pub fn abstract_shape_create_circle(radius: f64) -> *mut AbstractShape;

    #[cpp(func = "AbstractShape* abstract_shape_create_rectangle(double, double)")]
    pub fn abstract_shape_create_rectangle(width: f64, height: f64) -> *mut AbstractShape;
}
