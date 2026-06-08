use hicc::AbiClass;

hicc::cpp! {
    #include <iostream>
    #include <typeinfo>
    #include <cmath>

    #include "typeid_rtti.h"
}

hicc::import_class! {
    #[cpp(class = "Shape", destroy = "shape_delete")]
    pub class Shape {
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

    // 纯虚函数通过 C ABI 包装调用，避免 macOS ARM64 vtable 兼容问题
    #[cpp(func = "int shape_getType(Shape*)")]
    fn shape_getType(self_: *mut Shape) -> i32;

    #[cpp(func = "double shape_area(Shape*)")]
    fn shape_area(self_: *mut Shape) -> f64;
}

fn main() {
    use hicc::AbiClass;

    println!("=== 023_typeid_rtti - typeid 与 RTTI ===\n");

    let mut circle = unsafe { shape_new_circle(5.0).into_unique() };
    let mut rect = unsafe { shape_new_rectangle(4.0, 6.0).into_unique() };
    let mut triangle = unsafe { shape_new_triangle(3.0, 4.0).into_unique() };

    println!("\nUsing typeid to determine runtime type:");
    println!("Circle: type={}, name={}, area={:.2}",
        shape_getType(&circle.as_mut_ptr()),
        "Circle",
        shape_area(&circle.as_mut_ptr())
    );

    println!("Rectangle: type={}, area={:.2}",
        shape_getType(&rect.as_mut_ptr()),
        shape_area(&rect.as_mut_ptr())
    );

    println!("Triangle: type={}, area={:.2}",
        shape_getType(&triangle.as_mut_ptr()),
        shape_area(&triangle.as_mut_ptr())
    );

    drop(circle);
    drop(rect);
    drop(triangle);

    println!("\nRust FFI: typeid 变成类型枚举或字符串比较");
    println!("RTTI 信息在 FFI 边界丢失，需在 C++ 侧导出类型信息");
}

