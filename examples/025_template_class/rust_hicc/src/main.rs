hicc::cpp! {
    #include <iostream>
    #include <stack>

    #include "template_class.h"
}

hicc::import_class! {
    #[cpp(class = "IntStack", destroy = "intstack_delete")]
    pub class IntStack {
        #[cpp(method = "int size() const")]
        fn size(&self) -> i32;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "void push(int value)")]
        fn push(&mut self, value: i32);

        #[cpp(method = "int top() const")]
        fn top(&self) -> i32;

        #[cpp(method = "void pop()")]
        fn pop(&mut self);
    }
}

hicc::import_class! {
    #[cpp(class = "DoubleStack", destroy = "doublestack_delete")]
    pub class DoubleStack {
        #[cpp(method = "int size() const")]
        fn size(&self) -> i32;

        #[cpp(method = "bool empty() const")]
        fn empty(&self) -> bool;

        #[cpp(method = "void push(double value)")]
        fn push(&mut self, value: f64);

        #[cpp(method = "double top() const")]
        fn top(&self) -> f64;

        #[cpp(method = "void pop()")]
        fn pop(&mut self);
    }
}

hicc::import_lib! {
    #![link_name = "template_class"]

    class IntStack;
    class DoubleStack;

    #[cpp(func = "IntStack* intstack_new()")]
    fn intstack_new() -> IntStack;

    #[cpp(func = "DoubleStack* doublestack_new()")]
    fn doublestack_new() -> DoubleStack;
}

fn main() {
    println!("=== 025_template_class - 类模板 ===\n");

    // IntStack
    let mut int_stack = intstack_new();
    println!("IntStack empty: {}", int_stack.empty());

    int_stack.push(10);
    int_stack.push(20);
    int_stack.push(30);

    println!("IntStack size: {}", int_stack.size());
    println!("IntStack top: {}", int_stack.top());
    int_stack.pop();
    println!("After pop, top: {}", int_stack.top());

    println!();

    // DoubleStack
    let mut double_stack = doublestack_new();
    println!("DoubleStack empty: {}", double_stack.empty());

    double_stack.push(1.1);
    double_stack.push(2.2);
    double_stack.push(3.3);

    println!("DoubleStack size: {}", double_stack.size());
    println!("DoubleStack top: {}", double_stack.top());
    double_stack.pop();
    println!("After pop, top: {}", double_stack.top());

    println!("\nRust FFI: 类模板 = 为每种类型实例化独立结构");
    println!("Stack<int> -> IntStack");
    println!("Stack<double> -> DoubleStack");
}

