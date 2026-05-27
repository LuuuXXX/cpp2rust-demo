hicc::cpp! {
    #include <iostream>
    #include <cmath>

    class Point {
        int x;
        int y;
    public:
        Point(int x, int y) : x(x), y(y) {}
        ~Point() {}
        int getX() const { return x; }
        int getY() const { return y; }
        double getMagnitude() const {
    return std::sqrt(x * x + y * y);
}
        double getAngle() const {
    return std::atan2(y, x);
}
    };

    Point* point_new_xy(int x, int y) {
        return new Point(x, y);
    }

    Point* point_newPolar(double r, double theta) {
        int x = static_cast<int>(r * std::cos(theta));
        int y = static_cast<int>(r * std::sin(theta));
        std::cout << "Point created: polar(" << r << ", " << theta << ") -> xy(" << x << ", " << y << ")" << std::endl;
        return new Point(x, y);
    }

    void point_delete(Point* self) {
        delete self;
    }
}

hicc::import_class! {
    #[cpp(class = "Point")]
    class Point {
        #[cpp(method = "int getX() const")]
        fn get_x(&self) -> i32;

        #[cpp(method = "int getY() const")]
        fn get_y(&self) -> i32;

        #[cpp(method = "double getMagnitude() const")]
        fn get_magnitude(&self) -> f64;

        #[cpp(method = "double getAngle() const")]
        fn get_angle(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "class_constructor"]

    class Point;

    #[cpp(func = "Point* point_new_xy(int, int)")]
    fn point_new_xy(x: i32, y: i32) -> *mut Point;

    #[cpp(func = "Point* point_newPolar(double, double)")]
    fn point_new_polar(r: f64, theta: f64) -> *mut Point;

    #[cpp(func = "void point_delete(Point* self)")]
    unsafe fn point_delete(self_: *mut Point);
}

fn main() {
    // Use Cartesian coordinates constructor
    let p1 = point_new_xy(3, 4);
    unsafe {
        println!("Point 1: ({}, {})", p1.get_x(), p1.get_y());
        println!("  Magnitude: {}", p1.get_magnitude());
        println!("  Angle: {}", p1.get_angle());
        point_delete(&p1);
    }

    println!();

    // Use polar coordinates constructor
    let p2 = point_new_polar(5.0, 0.0);
    unsafe {
        println!("Point 2: ({}, {})", p2.get_x(), p2.get_y());
        println!("  Magnitude: {}", p2.get_magnitude());
        println!("  Angle: {}", p2.get_angle());
        point_delete(&p2);
    }

    println!("\nRust FFI: Multiple constructors work!");
}



