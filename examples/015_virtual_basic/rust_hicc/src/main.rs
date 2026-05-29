hicc::cpp! {
    #include <iostream>
    #include <cmath>
    #include <cstring>
    #include <string>

    #include "virtual_basic.h"
}

hicc::import_class! {
    #[cpp(class = "Shape", destroy = "shape_delete")]
    class Shape {
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Circle", destroy = "circle_delete")]
    class Circle {
        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;

        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "double getRadius() const")]
        fn get_radius(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "virtual_basic"]

    class Shape;
    class Circle;

    #[cpp(func = "Shape* shape_new()")]
    fn shape_new() -> Shape;

    #[cpp(func = "Circle* circle_new(double)")]
    fn circle_new(radius: f64) -> Circle;
}

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    println!("=== Virtual Function FFI with hicc ===\n");

    // Create Circle
    let circle = circle_new(5.0);

    println!("Circle name: {}", decode_cstr(circle.get_name()));
    println!("Circle radius: {}", circle.get_radius());
    println!("Circle area: {:.4}", circle.area());

    println!("\nRust FFI: Virtual functions work through hicc import_class!");
}

