hicc::cpp! {
    #include <iostream>
    #include <typeinfo>
    #include <cmath>

    class Shape {
    public:
        virtual ~Shape() = default;
        virtual int getType() const = 0;
        virtual const char* getTypeName() const = 0;
        virtual double area() const = 0;
    };

    class Circle : public Shape {
        double radius;
    public:
        Circle(double r);
        int getType() const override;
        const char* getTypeName() const override;
        double area() const override;
    };

    class Rectangle : public Shape {
        double width;
        double height;
    public:
        Rectangle(double w, double h);
        int getType() const override;
        const char* getTypeName() const override;
        double area() const override;
    };

    class Triangle : public Shape {
        double base;
        double height;
    public:
        Triangle(double b, double h);
        int getType() const override;
        const char* getTypeName() const override;
        double area() const override;
    };

    Circle::Circle(double r) : radius(r) {}

    int Circle::getType() const { return SHAPE_TYPE_CIRCLE; }

    const char* Circle::getTypeName() const { return "Circle"; }

    double Circle::area() const { return 3.14159 * radius * radius; }

    Rectangle::Rectangle(double w, double h) : width(w), height(h) {}

    int Rectangle::getType() const { return SHAPE_TYPE_RECTANGLE; }

    const char* Rectangle::getTypeName() const { return "Rectangle"; }

    double Rectangle::area() const { return width * height; }

    Triangle::Triangle(double b, double h) : base(b), height(h) {}

    int Triangle::getType() const { return SHAPE_TYPE_TRIANGLE; }

    const char* Triangle::getTypeName() const { return "Triangle"; }

    double Triangle::area() const { return 0.5 * base * height; }

    Shape* shape_new_circle(double radius) {
        return new Circle(radius);
    }

    Shape* shape_new_rectangle(double width, double height) {
        return new Rectangle(width, height);
    }

    Shape* shape_new_triangle(double base, double height) {
        return new Triangle(base, height);
    }

    void shape_delete(Shape* self) {
        if (self) {
            std::cout << "Deleting " << self->getTypeName() << std::endl;
            delete self;
        }
    }
}

hicc::import_class! {
    #[cpp(class = "Shape")]
    class Shape {
        #[cpp(method = "int getType() const")]
        fn get_type(&self) -> i32;

        #[cpp(method = "const char* getTypeName() const")]
        fn get_type_name(&self) -> *const u8;

        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "typeid_rtti"]

    class Shape;

    #[cpp(func = "Shape* shape_new_circle(double)")]
    fn shape_new_circle(radius: f64) -> *mut Shape;

    #[cpp(func = "Shape* shape_new_rectangle(double, double)")]
    fn shape_new_rectangle(width: f64, height: f64) -> *mut Shape;

    #[cpp(func = "Shape* shape_new_triangle(double, double)")]
    fn shape_new_triangle(base: f64, height: f64) -> *mut Shape;

    #[cpp(func = "void shape_delete(Shape* self)")]
    unsafe fn shape_delete(self_: *mut Shape);
}

fn main() {
    println!("=== 023_typeid_rtti - typeid 与 RTTI ===\n");

    let circle = shape_new_circle(5.0);
    let rect = shape_new_rectangle(4.0, 6.0);
    let triangle = shape_new_triangle(3.0, 4.0);

    println!("\nUsing typeid to determine runtime type:");
    println!("Circle: type={}, name={}, area={:.2}",
        shape_getType(&circle),
        "Circle",
        shape_area(&circle)
    );

    println!("Rectangle: type={}, area={:.2}",
        shape_getType(&rect),
        shape_area(&rect)
    );

    println!("Triangle: type={}, area={:.2}",
        shape_getType(&triangle),
        shape_area(&triangle)
    );

    unsafe {
        shape_delete(&circle);
        shape_delete(&rect);
        shape_delete(&triangle);
    }

    println!("\nRust FFI: typeid 变成类型枚举或字符串比较");
    println!("RTTI 信息在 FFI 边界丢失，需在 C++ 侧导出类型信息");
}



