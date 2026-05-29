hicc::cpp! {
    #include <iostream>
    #include <cmath>
    #include <cstring>

    AbstractShape* abstract_shape_create_circle(double radius) {
        return new Circle(radius);
    }

    AbstractShape* abstract_shape_create_rectangle(double width, double height) {
        return new Rectangle(width, height);
    }

    void abstract_shape_delete(AbstractShape* self) {
        if (self) {
            std::cout << "Deleting " << self->getName() << std::endl;
            delete self;
        }
    }
}

hicc::import_class! {
    #[interface]
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

fn main() {
    println!("=== Pure Virtual Function FFI with hicc ===\n");
    println!("Pure virtual functions (= 0) make a class abstract");
    println!("Cannot be instantiated directly in C++\n");

    // Create Circle (concrete implementation)
    let circle = unsafe { abstract_shape_create_circle(5.0) };

    // Create Rectangle (concrete implementation)
    let rectangle = unsafe { abstract_shape_create_rectangle(4.0, 6.0) };

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

    println!("\nRust FFI: Pure virtual functions work through hicc!");
}
