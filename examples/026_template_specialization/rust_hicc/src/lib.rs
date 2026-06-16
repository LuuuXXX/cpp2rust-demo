//! 026_template_specialization: 模板特化（通用模板 + std::string 全特化）。
//!
//! C++ 通用类模板 `ValueHolder<T>` 提供默认实现，`ValueHolder<std::string>` 为全特化，
//! 行为不同（`describe()` 输出长度信息）。模板/特化本身不可裸绑定，本示例将每个具体
//! 类型暴露为 idiomatic 命名空间类（`IntHolder`/`DoubleHolder` 走通用模板、`StringHolder`
//! 走特化版本），hicc 直出按普通类绑定方法与 `make_unique` 工厂，无需 extern-C shim。
//!
//! 注：`ValueHolder<std::string>` 的构造函数设为私有 + `friend StringHolder`，避免被
//! hicc 直出当作可独立实例化的普通类绑定（其类名带模板实参，无法裸用）。

hicc::cpp! {
    #include "template_specialization.h"
}

hicc::import_class! {
    #[cpp(class = "template_specialization_ns::IntHolder")]
    pub class IntHolder {
        #[cpp(method = "int get() const")]
        pub fn get(&self) -> i32;

        #[cpp(method = "const char* describe() const")]
        pub fn describe(&self) -> *const i8;

        pub fn new(value: i32) -> Self { int_holder_new(value) }
    }
}

hicc::import_class! {
    #[cpp(class = "template_specialization_ns::DoubleHolder")]
    pub class DoubleHolder {
        #[cpp(method = "double get() const")]
        pub fn get(&self) -> f64;

        #[cpp(method = "const char* describe() const")]
        pub fn describe(&self) -> *const i8;

        pub fn new(value: f64) -> Self { double_holder_new(value) }
    }
}

hicc::import_class! {
    #[cpp(class = "template_specialization_ns::StringHolder")]
    pub class StringHolder {
        #[cpp(method = "const char* get() const")]
        pub fn get(&self) -> *const i8;

        #[cpp(method = "const char* describe() const")]
        pub fn describe(&self) -> *const i8;

        pub fn new(value: *const i8) -> Self { string_holder_new(value) }
    }
}

hicc::import_lib! {
    #![link_name = "template_specialization"]

    #[cpp(func = "std::unique_ptr<template_specialization_ns::IntHolder> hicc::make_unique<template_specialization_ns::IntHolder, int>(int&&)")]
    pub fn int_holder_new(value: i32) -> IntHolder;

    #[cpp(func = "std::unique_ptr<template_specialization_ns::DoubleHolder> hicc::make_unique<template_specialization_ns::DoubleHolder, double>(double&&)")]
    pub fn double_holder_new(value: f64) -> DoubleHolder;

    #[cpp(func = "std::unique_ptr<template_specialization_ns::StringHolder> hicc::make_unique<template_specialization_ns::StringHolder, const char*>(const char*&&)")]
    pub fn string_holder_new(value: *const i8) -> StringHolder;
}
