// 015_virtual_basic 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「含虚函数及覆写的命名空间类」默认生成的 hicc 骨架。
// 基类 Shape 的虚函数 area() 与派生类 Circle 的覆写 area() 各自绑定；double 成员/构造
// 均可直出映射，本示例 `lib.rs` 与支架一致（无需手写补全）。

hicc::cpp! {
    #include "virtual_basic.h"
}

hicc::import_class! {
    #[cpp(class = "virtual_basic_ns::Shape")]
    pub class Shape {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        pub fn new() -> Self { shape_new() }
    }
}

hicc::import_class! {
    #[cpp(class = "virtual_basic_ns::Circle")]
    pub class Circle {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "double radius() const")]
        pub fn radius(&self) -> f64;

        pub fn new(r: f64) -> Self { circle_new(r) }
    }
}

hicc::import_lib! {
    #![link_name = "virtual_basic"]

    #[cpp(func = "std::unique_ptr<virtual_basic_ns::Shape> hicc::make_unique<virtual_basic_ns::Shape>()")]
    pub fn shape_new() -> Shape;

    #[cpp(func = "std::unique_ptr<virtual_basic_ns::Circle> hicc::make_unique<virtual_basic_ns::Circle, double>(double&&)")]
    pub fn circle_new(r: f64) -> Circle;
}
