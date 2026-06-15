// 此文件为 cpp2rust-demo 工具对 023_typeid_rtti 自动生成的支架黄金文件，
// 仅供 L1 golden 测试进行生成准确性验证。
//
// Direct 模式下，工具为 Circle/Rectangle/Triangle 生成 import_class! + make_unique 工厂，
// 使用 std::make_unique<T>(args) 模板函数绑定（无法直接通过 hicc 导出模板，
// 实际 lib.rs 用手动 C++ 包装函数替代）。
hicc::cpp! {
    #include <cmath>
    #include "typeid_rtti.h"
}

hicc::import_class! {
    #[cpp(class = "Circle")]
    pub class Circle {
        #[cpp(method = "int getType() const")]
        pub fn get_type(&self) -> i32;

        #[cpp(method = "const char* getTypeName() const")]
        pub fn get_type_name(&self) -> *const i8;

        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;
    }
}

hicc::import_class! {
    #[cpp(class = "Rectangle")]
    pub class Rectangle {
        #[cpp(method = "int getType() const")]
        pub fn get_type(&self) -> i32;

        #[cpp(method = "const char* getTypeName() const")]
        pub fn get_type_name(&self) -> *const i8;

        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;
    }
}

hicc::import_class! {
    #[cpp(class = "Triangle")]
    pub class Triangle {
        #[cpp(method = "int getType() const")]
        pub fn get_type(&self) -> i32;

        #[cpp(method = "const char* getTypeName() const")]
        pub fn get_type_name(&self) -> *const i8;

        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "typeid_rtti"]

    class Circle;
    class Rectangle;
    class Triangle;

    #[cpp(func = "std::unique_ptr<Circle> std::make_unique<Circle>(double)")]
    pub fn circle_new_with_r(r: f64) -> Circle;

    #[cpp(func = "std::unique_ptr<Rectangle> std::make_unique<Rectangle>(double, double)")]
    pub fn rectangle_new_2(w: f64, h: f64) -> Rectangle;

    #[cpp(func = "std::unique_ptr<Triangle> std::make_unique<Triangle>(double, double)")]
    pub fn triangle_new_2(b: f64, h: f64) -> Triangle;
}
