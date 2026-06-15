hicc::cpp! {
    #include <iostream>
    #include <cmath>
    #include <cstring>
    #include <string>

    #include "virtual_basic.h"

    std::unique_ptr<Shape> _cpp2rust_make_unique_shape_with_n(const char* n) { return std::make_unique<Shape>(n); }
    std::unique_ptr<Circle> _cpp2rust_make_unique_circle_with_r(double r) { return std::make_unique<Circle>(r); }
}

hicc::import_class! {
    #[cpp(class = "Shape")]
    pub class Shape {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Circle")]
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

    #[cpp(func = "std::unique_ptr<Shape> _cpp2rust_make_unique_shape_with_n(const char*)")]
    pub unsafe fn shape_new_with_n(n: *const i8) -> Shape;

    #[cpp(func = "std::unique_ptr<Circle> _cpp2rust_make_unique_circle_with_r(double)")]
    pub fn circle_new_with_r(r: f64) -> Circle;
}
