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
    }
}

hicc::import_lib! {
    #![link_name = "virtual_pure"]

    class AbstractShape;

    #[cpp(func = "AbstractShape* abstract_shape_create_circle(double)")]
    fn abstract_shape_create_circle(radius: f64) -> *mut AbstractShape;

    #[cpp(func = "AbstractShape* abstract_shape_create_rectangle(double, double)")]
    fn abstract_shape_create_rectangle(width: f64, height: f64) -> *mut AbstractShape;

    // 纯虚函数通过 C ABI 包装调用，避免 macOS ARM64 vtable 兼容问题
    #[cpp(func = "double abstract_shape_area(AbstractShape*)")]
    fn abstract_shape_area(self_: *mut AbstractShape) -> f64;

    #[cpp(func = "const char* abstract_shape_getName(AbstractShape*)")]
    fn abstract_shape_getName(self_: *mut AbstractShape) -> *const i8;
}

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    println!("=== Pure Virtual Function FFI with hicc ===\n");
    println!("Pure virtual functions (= 0) make a class abstract");
    println!("Cannot be instantiated directly in C++\n");

    // Create Circle (concrete implementation)
    let mut circle = abstract_shape_create_circle(5.0);

    // Create Rectangle (concrete implementation)
    let mut rectangle = abstract_shape_create_rectangle(4.0, 6.0);

    // Use polymorphism: same interface, different implementations
    println!("\n--- Using circle through abstract interface ---");
    println!("Shape: {}", decode_cstr(abstract_shape_getName(&circle.as_mut_ptr())));
    let area = abstract_shape_area(&circle.as_mut_ptr());
    println!("Area: {:.4}", area);

    println!("\n--- Using rectangle through abstract interface ---");
    println!("Shape: {}", decode_cstr(abstract_shape_getName(&rectangle.as_mut_ptr())));
    let area = abstract_shape_area(&rectangle.as_mut_ptr());
    println!("Area: {:.4}", area);

    println!("\n--- Polymorphic behavior demonstrated ---");
    // into_value() extracts the inner T (with no_destroy_methods), into_unique() switches
    // to destroy_methods so the subsequent drop triggers abstract_shape_delete, which in
    // turn calls delete and the C++ destructor — producing the two "Deleting X" lines each.
    unsafe { circle.into_value().into_unique() };
    unsafe { rectangle.into_value().into_unique() };

    println!("\nRust FFI: Pure virtual functions work through hicc!");
}
