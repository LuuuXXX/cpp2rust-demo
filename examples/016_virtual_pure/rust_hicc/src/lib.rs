hicc::cpp! {
    #include <iostream>
    #include <cmath>
    #include <cstring>

    #include "virtual_pure.h"
}

hicc::import_class! {
    #[cpp(class = "Circle")]
    pub class Circle {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Rectangle")]
    pub class Rectangle {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "virtual_pure"]

    class Circle;
    class Rectangle;

    #[cpp(func = "std::unique_ptr<Circle> std::make_unique<Circle>(double)")]
    pub fn circle_new_with_r(r: f64) -> Circle;

    #[cpp(func = "std::unique_ptr<Rectangle> std::make_unique<Rectangle>(double, double)")]
    pub fn rectangle_new_2(w: f64, h: f64) -> Rectangle;
}
