hicc::cpp! {
    #include <cmath>

    #include "typeid_rtti.h"

    std::unique_ptr<Circle> _cpp2rust_make_unique_circle_with_r(double r) { return std::make_unique<Circle>(r); }
    std::unique_ptr<Rectangle> _cpp2rust_make_unique_rectangle_2(double w, double h) { return std::make_unique<Rectangle>(w, h); }
    std::unique_ptr<Triangle> _cpp2rust_make_unique_triangle_2(double b, double h) { return std::make_unique<Triangle>(b, h); }
}

hicc::import_class! {
    #[cpp(class = "Circle")]
    pub class Circle {
        #[cpp(method = "int getType() const")]
        pub fn get_type(&self) -> i32;

        #[cpp(method = "const char* getTypeName() const")]
        pub fn get_type_name(&self) -> *const i8;

        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;
    }
}

hicc::import_class! {
    #[cpp(class = "Rectangle")]
    pub class Rectangle {
        #[cpp(method = "int getType() const")]
        pub fn get_type(&self) -> i32;

        #[cpp(method = "const char* getTypeName() const")]
        pub fn get_type_name(&self) -> *const i8;

        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;
    }
}

hicc::import_class! {
    #[cpp(class = "Triangle")]
    pub class Triangle {
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

    class Circle;
    class Rectangle;
    class Triangle;

    #[cpp(func = "std::unique_ptr<Circle> _cpp2rust_make_unique_circle_with_r(double)")]
    pub fn circle_new_with_r(r: f64) -> Circle;

    #[cpp(func = "std::unique_ptr<Rectangle> _cpp2rust_make_unique_rectangle_2(double, double)")]
    pub fn rectangle_new_2(w: f64, h: f64) -> Rectangle;

    #[cpp(func = "std::unique_ptr<Triangle> _cpp2rust_make_unique_triangle_2(double, double)")]
    pub fn triangle_new_2(b: f64, h: f64) -> Triangle;
}
