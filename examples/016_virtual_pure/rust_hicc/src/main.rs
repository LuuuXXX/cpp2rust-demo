hicc::cpp! {
    #include <iostream>
    #include <cmath>
    #include <cstring>

    class AbstractShape {
    public:
        virtual ~AbstractShape() = default;
        virtual double area() const = 0;
        virtual const char* getName() const = 0;
    };

    class Circle : public AbstractShape {
        double radius;
    public:
        Circle(double r);
        ~Circle() override;
        double area() const override;
        const char* getName() const override;
    };

    class Rectangle : public AbstractShape {
        double width;
        double height;
    public:
        Rectangle(double w, double h);
        ~Rectangle() override;
        double area() const override;
        const char* getName() const override;
    };

    Circle::Circle(double r) : radius(r) {}

    Circle::~Circle() {
        std::cout << "Deleting Circle" << std::endl;
    }

    double Circle::area() const {
        return 
              3.14159265358979323846 
                   * radius * radius;
    }

    const char* Circle::getName() const {
        return "Circle";
    }

    Rectangle::Rectangle(double w, double h) : width(w), height(h) {}

    Rectangle::~Rectangle() {
        std::cout << "Deleting Rectangle" << std::endl;
    }

    double Rectangle::area() const {
        return width * height;
    }

    const char* Rectangle::getName() const {
        return "Rectangle";
    }

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
    #[cpp(class = "AbstractShape")]
    class AbstractShape {
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const u8;
    }
}

hicc::import_lib! {
    #![link_name = "virtual_pure"]

    class AbstractShape;

    #[cpp(func = "AbstractShape* abstract_shape_create_circle(double)")]
    fn abstract_shape_create_circle(radius: f64) -> *mut AbstractShape;

    #[cpp(func = "AbstractShape* abstract_shape_create_rectangle(double, double)")]
    fn abstract_shape_create_rectangle(width: f64, height: f64) -> *mut AbstractShape;

    #[cpp(func = "void abstract_shape_delete(AbstractShape* self)")]
    unsafe fn abstract_shape_delete(self_: *mut AbstractShape);
}

fn decode_cstr(ptr: *const u8) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0;
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
        String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).to_string()
    }
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

    unsafe {
        abstract_shape_delete(&circle);
        abstract_shape_delete(&rectangle);
    }

    println!("\nRust FFI: Pure virtual functions work through hicc!");
}


