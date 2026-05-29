hicc::cpp! {
    #include <iostream>
    #include <typeinfo>
    #include <cmath>

    enum ShapeType {
        SHAPE_TYPE_CIRCLE = 0,
        SHAPE_TYPE_RECTANGLE = 1,
        SHAPE_TYPE_TRIANGLE = 2,
    };

    #include "typeid_rtti.h"
}

hicc::import_class! {
    #[interface]
    class Shape {
        #[cpp(method = "int getType() const")]
        fn get_type(&self) -> i32;

        #[cpp(method = "const char* getTypeName() const")]
        fn get_type_name(&self) -> *const i8;

        #[cpp(method = "double area() const")]
        fn area(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "typeid_rtti"]

    class Shape;

    #[cpp(func = "Shape* shape_new_circle(double)")]
    fn shape_new_circle(radius: f64) -> Shape;

    #[cpp(func = "Shape* shape_new_rectangle(double, double)")]
    fn shape_new_rectangle(width: f64, height: f64) -> Shape;

    #[cpp(func = "Shape* shape_new_triangle(double, double)")]
    fn shape_new_triangle(base: f64, height: f64) -> Shape;
}

fn main() {
    println!("=== 023_typeid_rtti - typeid 与 RTTI ===\n");

    let circle = shape_new_circle(5.0);
    let rect = shape_new_rectangle(4.0, 6.0);
    let triangle = shape_new_triangle(3.0, 4.0);

    println!("\nUsing typeid to determine runtime type:");
    println!("Circle: type={}, name={}, area={:.2}",
        circle.get_type(),
        "Circle",
        circle.area()
    );

    println!("Rectangle: type={}, area={:.2}",
        rect.get_type(),
        rect.area()
    );

    println!("Triangle: type={}, area={:.2}",
        triangle.get_type(),
        triangle.area()
    );

    println!("\nRust FFI: typeid 变成类型枚举或字符串比较");
    println!("RTTI 信息在 FFI 边界丢失，需在 C++ 侧导出类型信息");
}

