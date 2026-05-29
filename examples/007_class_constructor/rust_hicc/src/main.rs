hicc::cpp! {
    #include <iostream>
    #include <cmath>

    #include "class_constructor.h"
}

hicc::import_class! {
    #[cpp(class = "Point", destroy = "point_delete")]
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
    fn point_new_xy(x: i32, y: i32) -> Point;

    #[cpp(func = "Point* point_newPolar(double, double)")]
    fn point_new_polar(r: f64, theta: f64) -> Point;
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

