//! 015_virtual_basic: 虚函数与覆写（命名空间类 + make_unique 工厂）。
//!
//! hicc 直出：基类 `Shape` 声明虚函数 `area()`（默认 0），派生类
//! `Circle : public Shape` 以 `override` 覆写 `area()`（π·r²）。两类各自以
//! `import_class!` 直接绑定真实命名空间类；所有成员/构造均为 `double`/无参，
//! 可直出映射，故本示例 `lib.rs` 与工具默认支架（`lib_scaffold.rs`）一致。
//!
//! 虚函数的运行期分派由 C++ 负责：对 `Shape` 实例 `area()` 返回 0，对 `Circle`
//! 实例返回 π·r²，体现覆写生效。

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
