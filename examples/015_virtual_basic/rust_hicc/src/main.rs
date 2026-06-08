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
        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Circle", destroy = "circle_delete")]
    pub class Circle {
        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;

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

    // 虚函数通过 C ABI 包装调用，避免 macOS ARM64 vtable 兼容问题
    #[cpp(func = "double circle_area(Circle*)")]
    fn circle_area(self_: *mut Circle) -> f64;
}

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    use hicc::AbiClass;

    println!("=== Virtual Function FFI with hicc ===\n");

    // Create Circle
    let mut circle = circle_new(5.0);

    println!("Circle name: {}", decode_cstr(circle.get_name()));
    println!("Circle radius: {}", circle.get_radius());
    println!("Circle area: {:.4}", circle_area(&circle.as_mut_ptr()));

    println!("\nRust FFI: Virtual functions work through hicc import_class!");
}

