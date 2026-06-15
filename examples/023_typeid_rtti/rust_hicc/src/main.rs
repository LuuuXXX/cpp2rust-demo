use typeid_rtti::*;

fn decode_cstr(ptr: *const i8) -> String {
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

fn main() {
    println!("=== 023_typeid_rtti - typeid 与 RTTI ===\n");

    let circle = circle_new_with_r(5.0);
    let rect = rectangle_new_2(4.0, 6.0);
    let triangle = triangle_new_2(3.0, 4.0);

    println!("Using getType/getTypeName to determine runtime type:");
    println!("Circle: type={}, name={}, area={:.2}",
        circle.get_type(),
        decode_cstr(circle.get_type_name()),
        circle.area()
    );

    println!("Rectangle: type={}, name={}, area={:.2}",
        rect.get_type(),
        decode_cstr(rect.get_type_name()),
        rect.area()
    );

    println!("Triangle: type={}, name={}, area={:.2}",
        triangle.get_type(),
        decode_cstr(triangle.get_type_name()),
        triangle.area()
    );

    println!("\nRust FFI: typeid 变成类型枚举或字符串比较");
    println!("RTTI 信息在 FFI 边界丢失，需在 C++ 侧导出类型信息");
}

