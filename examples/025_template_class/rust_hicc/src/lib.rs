// AbiClass is required by the `class!` macro expansion below.
use hicc::AbiClass;

hicc::cpp! {
    #include <iostream>
    #include <stack>

    #include "template_class.h"
}

hicc::import_class! {
    #[cpp(class = "IntStack", destroy = "intstack_delete")]
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
    #[cpp(class = "DoubleStack", destroy = "doublestack_delete")]
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

    #[cpp(func = "IntStack* intstack_new()")]
    pub fn intstack_new() -> IntStack;

    #[cpp(func = "DoubleStack* doublestack_new()")]
    pub fn doublestack_new() -> DoubleStack;
}
