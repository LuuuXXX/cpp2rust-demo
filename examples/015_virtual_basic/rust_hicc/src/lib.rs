hicc::cpp! {
    #include <iostream>
    #include <cmath>
    #include <cstring>
    #include <string>

    #include "virtual_basic.h"
}

hicc::import_class! {
    #[cpp(class = "Shape", destroy = "shape_delete")]
    pub class Shape {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Circle", destroy = "circle_delete")]
    pub class Circle {
        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;

        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "double getRadius() const")]
        pub fn get_radius(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "virtual_basic"]

    class Shape;
    class Circle;

    #[cpp(func = "Shape* shape_new()")]
    pub fn shape_new() -> Shape;

    #[cpp(func = "Circle* circle_new(double)")]
    pub fn circle_new(radius: f64) -> Circle;
}
