// 此文件为 cpp2rust-demo 工具对 025_template_class 自动生成的支架黄金文件，
// 仅供 L1 golden 测试（test_025_template_class）校验工具默认产物的生成准确性。

hicc::cpp! {
    #include <iostream>
    #include <stack>

    #include "template_class.h"
}

hicc::import_class! {
    #[cpp(class = "IntStack")]
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
    }
}

hicc::import_class! {
    #[cpp(class = "DoubleStack")]
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
    }
}

hicc::import_lib! {
    #![link_name = "template_class"]

    class IntStack;
    class DoubleStack;

    #[cpp(func = "std::unique_ptr<IntStack> hicc::make_unique<IntStack>()")]
    pub fn int_stack_new() -> IntStack;

    #[cpp(func = "std::unique_ptr<DoubleStack> hicc::make_unique<DoubleStack>()")]
    pub fn double_stack_new() -> DoubleStack;
}
