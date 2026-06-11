use hicc::AbiClass;
use typeid_rtti::*;

fn main() {
    println!("=== 023_typeid_rtti - typeid 与 RTTI ===\n");

    let circle = unsafe { shape_new_circle(5.0).into_unique() };
    let rect = unsafe { shape_new_rectangle(4.0, 6.0).into_unique() };
    let triangle = unsafe { shape_new_triangle(3.0, 4.0).into_unique() };

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

    drop(circle);
    drop(rect);
    drop(triangle);

    println!("\nRust FFI: typeid 变成类型枚举或字符串比较");
    println!("RTTI 信息在 FFI 边界丢失，需在 C++ 侧导出类型信息");
}

