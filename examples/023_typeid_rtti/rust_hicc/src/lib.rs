//! 023_typeid_rtti: RTTI / typeid（多态命名空间类 + make_unique 工厂）。
//!
//! hicc 直出：抽象基类 `Shape` 声明纯虚 `area() const = 0`，无公有构造、不可实例化，
//! 被工具跳过；具体类 `Circle`/`Rectangle`/`Triangle` 各自以 `import_class!` 直接绑定
//! （见 `lib_scaffold.rs`）。RTTI 不在直出范围内：本文件用 `hicc::cpp!` 命名包装函数
//! 对每个具体对象经**基类引用**调用 `typeid(...).name()`，再以 `#[cpp(func = ...)]`
//! 绑定为关联方法 `runtime_type_name`，从而演示「经基类引用取回动态类型」的 RTTI 行为。

hicc::cpp! {
    #include "typeid_rtti.h"
    #include <typeinfo>

    using typeid_rtti_ns::Shape;
    using typeid_rtti_ns::Circle;
    using typeid_rtti_ns::Rectangle;
    using typeid_rtti_ns::Triangle;

    const char* circle_runtime_type(const Circle* self) {
        const Shape& s = *self;          // 上行为基类引用
        return typeid(s).name();         // RTTI 取回动态类型名（Circle）
    }
    const char* rectangle_runtime_type(const Rectangle* self) {
        const Shape& s = *self;
        return typeid(s).name();
    }
    const char* triangle_runtime_type(const Triangle* self) {
        const Shape& s = *self;
        return typeid(s).name();
    }
}

hicc::import_class! {
    #[cpp(class = "typeid_rtti_ns::Circle")]
    pub class Circle {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "double radius() const")]
        pub fn radius(&self) -> f64;

        // RTTI：经基类引用取回的动态类型名（typeid），返回 C 字符串指针
        #[cpp(func = "const char* circle_runtime_type(const typeid_rtti_ns::Circle*)")]
        pub fn runtime_type_name(&self) -> *const i8;

        pub fn new(r: f64) -> Self { circle_new(r) }
    }
}

hicc::import_class! {
    #[cpp(class = "typeid_rtti_ns::Rectangle")]
    pub class Rectangle {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(func = "const char* rectangle_runtime_type(const typeid_rtti_ns::Rectangle*)")]
        pub fn runtime_type_name(&self) -> *const i8;

        pub fn new(w: f64, h: f64) -> Self { rectangle_new(w, h) }
    }
}

hicc::import_class! {
    #[cpp(class = "typeid_rtti_ns::Triangle")]
    pub class Triangle {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(func = "const char* triangle_runtime_type(const typeid_rtti_ns::Triangle*)")]
        pub fn runtime_type_name(&self) -> *const i8;

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
