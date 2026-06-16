//! 019_operator_overload: 运算符重载（命名空间类 + make_unique 工厂）。
//!
//! hicc 直出会**跳过 operator 重载**（`operator+` 等不进 `import_class!`），普通方法
//! `value()`/`compare()` 与构造工厂照常生成（见 `lib_scaffold.rs`）。运算符则在手写
//! `lib.rs` 中以 `hicc::cpp!` 命名包装函数补全：每个运算符包成一个具名 C++ 函数，
//! 返回 `std::unique_ptr<Number>`（或就地修改 `*self`），再用 `#[cpp(func = ...)]`
//! 绑定为 `Number` 的关联方法，并据此实现 Rust 的 `std::ops` 运算符 trait。

hicc::cpp! {
    #include "operator_overload.h"
    #include <memory>

    using operator_overload_ns::Number;

    std::unique_ptr<Number> number_op_add(const Number* self, const Number& other) {
        return std::make_unique<Number>(*self + other);
    }
    std::unique_ptr<Number> number_op_sub(const Number* self, const Number& other) {
        return std::make_unique<Number>(*self - other);
    }
    std::unique_ptr<Number> number_op_mul(const Number* self, const Number& other) {
        return std::make_unique<Number>(*self * other);
    }
    std::unique_ptr<Number> number_op_div(const Number* self, const Number& other) {
        return std::make_unique<Number>(*self / other);
    }
    std::unique_ptr<Number> number_op_neg(const Number* self) {
        return std::make_unique<Number>(-(*self));
    }
    void number_op_inc(Number* self) { ++(*self); }
    void number_op_dec(Number* self) { --(*self); }
    void number_op_add_assign(Number* self, const Number& other) { *self += other; }
    void number_op_sub_assign(Number* self, const Number& other) { *self -= other; }
}

hicc::import_class! {
    #[cpp(class = "operator_overload_ns::Number")]
    pub class Number {
        #[cpp(method = "int value() const")]
        pub fn value(&self) -> i32;

        #[cpp(method = "int compare(const operator_overload_ns::Number & other) const")]
        pub fn compare(&self, other: &Number) -> i32;

        // 运算符重载：经 hicc::cpp! 命名包装函数绑定为关联方法
        #[cpp(func = "std::unique_ptr<operator_overload_ns::Number> number_op_add(const operator_overload_ns::Number*, const operator_overload_ns::Number&)")]
        pub fn op_add(&self, other: &Number) -> Number;

        #[cpp(func = "std::unique_ptr<operator_overload_ns::Number> number_op_sub(const operator_overload_ns::Number*, const operator_overload_ns::Number&)")]
        pub fn op_sub(&self, other: &Number) -> Number;

        #[cpp(func = "std::unique_ptr<operator_overload_ns::Number> number_op_mul(const operator_overload_ns::Number*, const operator_overload_ns::Number&)")]
        pub fn op_mul(&self, other: &Number) -> Number;

        #[cpp(func = "std::unique_ptr<operator_overload_ns::Number> number_op_div(const operator_overload_ns::Number*, const operator_overload_ns::Number&)")]
        pub fn op_div(&self, other: &Number) -> Number;

        #[cpp(func = "std::unique_ptr<operator_overload_ns::Number> number_op_neg(const operator_overload_ns::Number*)")]
        pub fn op_neg(&self) -> Number;

        #[cpp(func = "void number_op_inc(operator_overload_ns::Number*)")]
        pub fn increment(&mut self);

        #[cpp(func = "void number_op_dec(operator_overload_ns::Number*)")]
        pub fn decrement(&mut self);

        #[cpp(func = "void number_op_add_assign(operator_overload_ns::Number*, const operator_overload_ns::Number&)")]
        pub fn add_assign(&mut self, other: &Number);

        #[cpp(func = "void number_op_sub_assign(operator_overload_ns::Number*, const operator_overload_ns::Number&)")]
        pub fn sub_assign(&mut self, other: &Number);

        pub fn new(v: i32) -> Self { number_new(v) }
    }
}

hicc::import_lib! {
    #![link_name = "operator_overload"]

    #[cpp(func = "std::unique_ptr<operator_overload_ns::Number> hicc::make_unique<operator_overload_ns::Number, int>(int&&)")]
    pub fn number_new(v: i32) -> Number;
}
