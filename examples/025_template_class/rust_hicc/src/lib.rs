//! 025_template_class: 类模板（命名空间模板 Stack<T> + 具体实例化类）。
//!
//! C++ 类模板 `Stack<T>` 是「蓝图」，须按具体类型实例化（`Stack<int>` …）才成为
//! 可链接的具体类型。本示例将每个具体类型暴露为 idiomatic 命名空间类
//! （`IntStack` / `DoubleStack`，内部复用 `Stack<T>`），hicc 直出按普通类绑定其
//! 方法与 `make_unique` 构造工厂，无需任何 extern-C shim。

hicc::cpp! {
    #include "template_class.h"
}

hicc::import_class! {
    #[cpp(class = "template_class_ns::IntStack")]
    pub class IntStack {
        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        #[cpp(method = "bool empty() const")]
        pub fn empty(&self) -> bool;

        #[cpp(method = "void push(int value)")]
        pub fn push(&mut self, value: i32);

        #[cpp(method = "int top() const")]
        pub fn top(&self) -> i32;

        #[cpp(method = "void pop()")]
        pub fn pop(&mut self);

        pub fn new() -> Self { int_stack_new() }
    }
}

hicc::import_class! {
    #[cpp(class = "template_class_ns::DoubleStack")]
    pub class DoubleStack {
        #[cpp(method = "int size() const")]
        pub fn size(&self) -> i32;

        #[cpp(method = "bool empty() const")]
        pub fn empty(&self) -> bool;

        #[cpp(method = "void push(double value)")]
        pub fn push(&mut self, value: f64);

        #[cpp(method = "double top() const")]
        pub fn top(&self) -> f64;

        #[cpp(method = "void pop()")]
        pub fn pop(&mut self);

        pub fn new() -> Self { double_stack_new() }
    }
}

hicc::import_lib! {
    #![link_name = "template_class"]

    #[cpp(func = "std::unique_ptr<template_class_ns::IntStack> hicc::make_unique<template_class_ns::IntStack>()")]
    pub fn int_stack_new() -> IntStack;

    #[cpp(func = "std::unique_ptr<template_class_ns::DoubleStack> hicc::make_unique<template_class_ns::DoubleStack>()")]
    pub fn double_stack_new() -> DoubleStack;
}
