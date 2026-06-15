hicc::cpp! {
    #include <iostream>
    #include <cmath>

    #include "class_constructor.h"
}

hicc::import_class! {
    #[cpp(class = "Point")]
    pub class Point {
        #[cpp(method = "int getX() const")]
        pub fn get_x(&self) -> i32;

        #[cpp(method = "int getY() const")]
        pub fn get_y(&self) -> i32;

        #[cpp(method = "double getMagnitude() const")]
        pub fn get_magnitude(&self) -> f64;

        #[cpp(method = "double getAngle() const")]
        pub fn get_angle(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "class_constructor"]

    class Point;

    #[cpp(func = "std::unique_ptr<Point> std::make_unique<Point>(int, int)")]
    pub fn point_new_2(x: i32, y: i32) -> Point;
}
