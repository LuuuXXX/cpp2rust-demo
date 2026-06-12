hicc::cpp! {
    #include <iostream>
    #include <cmath>

    #include "class_constructor.h"
}

hicc::import_class! {
    #[cpp(class = "Point", destroy = "point_delete")]
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

    #[cpp(func = "Point* point_new_xy(int, int)")]
    pub fn point_new_xy(x: i32, y: i32) -> Point;

    #[cpp(func = "Point* point_newPolar(double, double)")]
    pub fn point_new_polar(r: f64, theta: f64) -> Point;
}
