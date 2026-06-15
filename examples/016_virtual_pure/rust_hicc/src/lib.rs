hicc::cpp! {
    #include <iostream>
    #include <cmath>
    #include <cstring>

    #include "virtual_pure.h"

    std::unique_ptr<Circle> _cpp2rust_make_unique_circle_with_r(double r) { return std::make_unique<Circle>(r); }
    std::unique_ptr<Rectangle> _cpp2rust_make_unique_rectangle_2(double w, double h) { return std::make_unique<Rectangle>(w, h); }
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

    #[cpp(func = "std::unique_ptr<Circle> _cpp2rust_make_unique_circle_with_r(double)")]
    pub fn circle_new_with_r(r: f64) -> Circle;

    #[cpp(func = "std::unique_ptr<Rectangle> _cpp2rust_make_unique_rectangle_2(double, double)")]
    pub fn rectangle_new_2(w: f64, h: f64) -> Rectangle;
}
