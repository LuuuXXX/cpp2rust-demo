// 023_typeid_rtti 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「多态抽象基类 + 具体实现」默认生成的 hicc 骨架。
// 抽象基类 Shape 无公有构造（不可实例化），被工具跳过；具体类 Circle/Rectangle/Triangle
// 各自绑定 area()/构造工厂。RTTI（typeid）由手写 `lib.rs` 经 hicc::cpp! 命名包装补全。

hicc::cpp! {
    #include "typeid_rtti.h"
}

hicc::import_class! {
    #[cpp(class = "typeid_rtti_ns::Circle")]
    pub class Circle {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "double radius() const")]
        pub fn radius(&self) -> f64;

        pub fn new(r: f64) -> Self { circle_new(r) }
    }
}

hicc::import_class! {
    #[cpp(class = "typeid_rtti_ns::Rectangle")]
    pub class Rectangle {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        pub fn new(w: f64, h: f64) -> Self { rectangle_new(w, h) }
    }
}

hicc::import_class! {
    #[cpp(class = "typeid_rtti_ns::Triangle")]
    pub class Triangle {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        pub fn new(b: f64, h: f64) -> Self { triangle_new(b, h) }
    }
}

hicc::import_lib! {
    #![link_name = "typeid_rtti"]

    #[cpp(func = "std::unique_ptr<typeid_rtti_ns::Circle> hicc::make_unique<typeid_rtti_ns::Circle, double>(double&&)")]
    pub fn circle_new(r: f64) -> Circle;

    #[cpp(func = "std::unique_ptr<typeid_rtti_ns::Rectangle> hicc::make_unique<typeid_rtti_ns::Rectangle, double, double>(double&&, double&&)")]
    pub fn rectangle_new(w: f64, h: f64) -> Rectangle;

    #[cpp(func = "std::unique_ptr<typeid_rtti_ns::Triangle> hicc::make_unique<typeid_rtti_ns::Triangle, double, double>(double&&, double&&)")]
    pub fn triangle_new(b: f64, h: f64) -> Triangle;
}
