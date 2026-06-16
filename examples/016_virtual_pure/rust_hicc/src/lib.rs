//! 016_virtual_pure: 纯虚接口与具体实现（命名空间类 + make_unique 工厂）。
//!
//! hicc 直出：抽象基类 `AbstractShape` 声明纯虚函数 `area() const = 0`，不可实例化，
//! 工具因其无公有构造而跳过（不生成绑定）；具体类 `Circle`/`Rectangle` 实现接口并各自
//! 以 `import_class!` 直接绑定真实命名空间类。所有成员/构造均为 `double`，可直出映射，
//! 故本示例 `lib.rs` 与工具默认支架（`lib_scaffold.rs`）一致（无需手写补全）。

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
