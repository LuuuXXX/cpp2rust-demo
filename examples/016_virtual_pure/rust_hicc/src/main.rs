use hicc::AbiClass;

hicc::cpp! {
    #include <iostream>
    #include <cmath>
    #include <cstring>

    #include "virtual_pure.h"
}

hicc::import_class! {
    #[cpp(class = "AbstractShape", destroy = "abstract_shape_delete")]
    class AbstractShape {
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "virtual_pure"]

    class AbstractShape;

    #[cpp(func = "AbstractShape* abstract_shape_create_circle(double)")]
    fn abstract_shape_create_circle(radius: f64) -> *mut AbstractShape;

    #[cpp(func = "AbstractShape* abstract_shape_create_rectangle(double, double)")]
    fn abstract_shape_create_rectangle(width: f64, height: f64) -> *mut AbstractShape;
}

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    println!("=== Pure Virtual Function FFI with hicc ===\n");
    println!("Pure virtual functions (= 0) make a class abstract");
    println!("Cannot be instantiated directly in C++\n");

    // Create Circle (concrete implementation)
    let circle = abstract_shape_create_circle(5.0);

    // Create Rectangle (concrete implementation)
    let rectangle = abstract_shape_create_rectangle(4.0, 6.0);

    // Use polymorphism: same interface, different implementations
    println!("\n--- Using circle through abstract interface ---");
    println!("Shape: {}", decode_cstr(circle.get_name()));
    let area = circle.area();
    println!("Area: {:.4}", area);

    println!("\n--- Using rectangle through abstract interface ---");
    println!("Shape: {}", decode_cstr(rectangle.get_name()));
    let area = rectangle.area();
    println!("Area: {:.4}", area);

    println!("\n--- Polymorphic behavior demonstrated ---");
    // into_value() extracts the inner T (with no_destroy_methods), into_unique() switches
    // to destroy_methods so the subsequent drop triggers abstract_shape_delete, which in
    // turn calls delete and the C++ destructor — producing the two "Deleting X" lines each.
    unsafe { circle.into_value().into_unique() };
    unsafe { rectangle.into_value().into_unique() };

    println!("\nRust FFI: Pure virtual functions work through hicc!");
}
