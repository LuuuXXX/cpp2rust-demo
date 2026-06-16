// 016_virtual_pure 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「纯虚接口 + 具体实现」默认生成的 hicc 骨架。
// 抽象基类 AbstractShape 无公有构造（不可实例化），被工具跳过，不生成绑定；
// 具体类 Circle/Rectangle 各自绑定，double 成员/构造可直出，`lib.rs` 与支架一致。

hicc::cpp! {
    #include "virtual_pure.h"
}

hicc::import_class! {
    #[cpp(class = "virtual_pure_ns::Circle")]
    pub class Circle {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "double radius() const")]
        pub fn radius(&self) -> f64;

        pub fn new(r: f64) -> Self { circle_new(r) }
    }
}

hicc::import_class! {
    #[cpp(class = "virtual_pure_ns::Rectangle")]
    pub class Rectangle {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        pub fn new(w: f64, h: f64) -> Self { rectangle_new(w, h) }
    }
}

hicc::import_lib! {
    #![link_name = "virtual_pure"]

    #[cpp(func = "std::unique_ptr<virtual_pure_ns::Circle> hicc::make_unique<virtual_pure_ns::Circle, double>(double&&)")]
    pub fn circle_new(r: f64) -> Circle;

    #[cpp(func = "std::unique_ptr<virtual_pure_ns::Rectangle> hicc::make_unique<virtual_pure_ns::Rectangle, double, double>(double&&, double&&)")]
    pub fn rectangle_new(w: f64, h: f64) -> Rectangle;
}
