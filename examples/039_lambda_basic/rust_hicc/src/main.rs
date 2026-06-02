hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <functional>
    #include <algorithm>

    #include "lambda_basic.h"

    typedef int (*IntBinaryOp)(int, int);
}

hicc::import_class! {
    #[cpp(class = "LambdaWrapper", destroy = "lambda_wrapper_delete")]
    pub class LambdaWrapper {
        #[cpp(method = "int invoke(int a, int b)")]
        fn invoke(&mut self, a: i32, b: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "StateLambda", destroy = "state_lambda_delete")]
    pub class StateLambda {
        #[cpp(method = "int get_value() const")]
        fn get_value(&self) -> i32;

        #[cpp(method = "int add(int delta)")]
        fn add(&mut self, delta: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Comparator", destroy = "comparator_delete")]
    pub class Comparator {
        #[cpp(method = "int compare(int a, int b) const")]
        fn compare(&self, a: i32, b: i32) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "lambda_basic"]

    class LambdaWrapper;
    class StateLambda;
    class Comparator;

    #[cpp(func = "StateLambda* state_lambda_new(int)")]
    fn state_lambda_new(initial_value: i32) -> StateLambda;

    #[cpp(func = "Comparator* comparator_new_add()")]
    fn comparator_new_add() -> Comparator;

    #[cpp(func = "int add_impl(int, int)")]
    fn add_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "int multiply_impl(int, int)")]
    fn multiply_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "int max_impl(int, int)")]
    fn max_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "LambdaWrapper* make_add_lambda()")]
    fn make_add_lambda() -> *mut LambdaWrapper;

    #[cpp(func = "LambdaWrapper* make_multiply_lambda()")]
    fn make_multiply_lambda() -> *mut LambdaWrapper;

    #[cpp(func = "LambdaWrapper* make_max_lambda()")]
    fn make_max_lambda() -> *mut LambdaWrapper;
}

fn main() {
    println!("=== 039_lambda_basic - Lambda 表达式 ===\n");

    println!("--- Direct function calls ---");
    println!("add_impl(3, 4) = {}", add_impl(3, 4));
    println!("multiply_impl(3, 4) = {}", multiply_impl(3, 4));
    println!("max_impl(3, 4) = {}", max_impl(3, 4));

    println!("\n--- LambdaWrapper Demo ---");
    let mut add_wrapper = make_add_lambda();
    println!("add invoke(5, 6) = {}", add_wrapper.invoke(5, 6));

    let mut mul_wrapper = make_multiply_lambda();
    println!("multiply invoke(5, 6) = {}", mul_wrapper.invoke(5, 6));

    println!("\n--- StateLambda Demo ---");
    let mut state = state_lambda_new(10);
    println!("initial value = {}", state.get_value());
    println!("add(5) = {}", state.add(5));
    println!("add(3) = {}", state.add(3));

    println!("\n--- Comparator Demo ---");
    let mut cmp = comparator_new_add();
    println!("compare(2, 3) = {}", cmp.compare(2, 3));

    println!("\nRust FFI: Lambda 表达式映射");
    println!("1. 函数指针可以通过 FFI 传递");
    println!("2. 捕获状态的 lambda 需要包装在类中");
    println!("3. 此示例展示基本的类封装模式");
}

