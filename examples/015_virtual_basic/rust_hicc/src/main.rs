hicc::cpp! {
    #include <iostream>
    #include <cmath>
    #include <cstring>
    #include <string>

    class Shape {
    protected:
        std::string name;
    public:
        Shape(const char* n);
        virtual ~Shape();
        virtual double area() const;
        const char* getName() const;
    };

    class Circle : public Shape {
        double radius;
    public:
        Circle(double r);
        ~Circle() override;
        double area() const override;
        double getRadius() const;
    };

    Shape::Shape(const char* n) : name(n) {}

    Shape::~Shape() {}

    double Shape::area() const {
        return 0.0;
    }

    const char* Shape::getName() const {
        return name.c_str();
    }

    Circle::Circle(double r) : Shape("Circle"), radius(r) {}

    Circle::~Circle() {}

    double Circle::area() const {
        return 
              3.14159265358979323846 
                   * radius * radius;
    }

    double Circle::getRadius() const {
        return radius;
    }

    Shape* shape_new() {
        return new Shape("Shape");
    }

    void shape_delete(Shape* self) {
        delete self;
    }

    Circle* circle_new(double radius) {
        return new Circle(radius);
    }

    void circle_delete(Circle* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "Shape")]
    class Shape {
        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;

        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const u8;
    }
}

hicc::import_class! {
    #[cpp(class = "Circle")]
    class Circle {
        #[cpp(method = "const char* getName() const")]
        fn get_name(&self) -> *const u8;

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
    fn shape_new() -> *mut Shape;

    #[cpp(func = "void shape_delete(Shape* self)")]
    unsafe fn shape_delete(self_: *mut Shape);

    #[cpp(func = "Circle* circle_new(double)")]
    fn circle_new(radius: f64) -> *mut Circle;

    #[cpp(func = "void circle_delete(Circle* self)")]
    unsafe fn circle_delete(self_: *mut Circle);
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
    println!("=== Virtual Function FFI with hicc ===\n");

    // Create Circle
    let circle = circle_new(5.0);

    println!("Circle name: {}", decode_cstr(circle.get_name()));
    println!("Circle radius: {}", circle.get_radius());
    println!("Circle area: {:.4}", circle.area());

    unsafe {
        circle_delete(&circle);
    }

    println!("\nRust FFI: Virtual functions work through hicc import_class!");
}


