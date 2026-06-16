//! 046_constexpr_basic: constexpr 构造与模板函数（命名空间类直接持有标量）。
//!
//! `ConstexprPoint` 演示 constexpr 构造函数与内联访问方法，`fibonacci<10>` / `array_size`
//! 演示编译期值经命名空间自由函数导出。hicc 直出无需 extern-C 不透明指针 + `*_delete`，析构由 Rust `Drop` 自动完成。

hicc::cpp! {
    #include "constexpr_basic.h"
}

hicc::import_class! {
    #[cpp(class = "constexpr_basic_ns::ConstexprPoint")]
    pub class ConstexprPoint {
        #[cpp(method = "int x() const")]
        pub fn x(&self) -> i32;

        #[cpp(method = "int y() const")]
        pub fn y(&self) -> i32;

        #[cpp(method = "int manhattan_distance() const")]
        pub fn manhattan_distance(&self) -> i32;

        pub fn new(x: i32, y: i32) -> Self { constexpr_point_new(x, y) }
    }
}

hicc::import_lib! {
    #![link_name = "constexpr_basic"]

    #[cpp(func = "std::unique_ptr<constexpr_basic_ns::ConstexprPoint> hicc::make_unique<constexpr_basic_ns::ConstexprPoint, int, int>(int&&, int&&)")]
    pub fn constexpr_point_new(x: i32, y: i32) -> ConstexprPoint;

    #[cpp(func = "int constexpr_basic_ns::fibonacci_10()")]
    pub fn fibonacci_10() -> i32;

    #[cpp(func = "int constexpr_basic_ns::array_size()")]
    pub fn array_size() -> i32;
}
