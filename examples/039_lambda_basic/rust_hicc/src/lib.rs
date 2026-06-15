hicc::cpp! {
    #include <iostream>
    #include <functional>
    #include <algorithm>

    #include "lambda_basic.h"

    int add_impl(int, int);
    int multiply_impl(int, int);
    int max_impl(int, int);
    int apply_operation(int, int, int (*)(int, int));
    int apply_twice(int, int (*)(int, int));

    std::unique_ptr<LambdaWrapper> _cpp2rust_make_unique_lambda_wrapper_with_fn(int (*fn)(int, int)) { return std::make_unique<LambdaWrapper>(fn); }
    std::unique_ptr<StateLambda> _cpp2rust_make_unique_state_lambda_with_initial_value(int initial_value) { return std::make_unique<StateLambda>(initial_value); }
    std::unique_ptr<Comparator> _cpp2rust_make_unique_comparator_with_cmp(int (*cmp)(int, int)) { return std::make_unique<Comparator>(cmp); }
    std::unique_ptr<Comparator> _cpp2rust_make_unique_comparator_new_add() { return std::make_unique<Comparator>(add_impl); }
    std::unique_ptr<LambdaWrapper> _cpp2rust_make_unique_lambda_wrapper_new_add() { return std::make_unique<LambdaWrapper>(add_impl); }
    std::unique_ptr<LambdaWrapper> _cpp2rust_make_unique_lambda_wrapper_new_multiply() { return std::make_unique<LambdaWrapper>(multiply_impl); }
    std::unique_ptr<LambdaWrapper> _cpp2rust_make_unique_lambda_wrapper_new_max() { return std::make_unique<LambdaWrapper>(max_impl); }
}

hicc::import_class! {
    #[cpp(class = "LambdaWrapper")]
    pub class LambdaWrapper {
        #[cpp(method = "int invoke(int a, int b)")]
        pub fn invoke(&mut self, a: i32, b: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "StateLambda")]
    pub class StateLambda {
        #[cpp(method = "int get_value() const")]
        pub fn get_value(&self) -> i32;

        #[cpp(method = "int add(int delta)")]
        pub fn add(&mut self, delta: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Comparator")]
    pub class Comparator {
        #[cpp(method = "int compare(int a, int b) const")]
        pub fn compare(&self, a: i32, b: i32) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "lambda_basic"]

    class LambdaWrapper;
    class StateLambda;
    class Comparator;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "std::unique_ptr<LambdaWrapper> _cpp2rust_make_unique_lambda_wrapper_with_fn(int (*)(int, int))")]
    pub unsafe fn lambda_wrapper_new(fn_: unsafe extern "C" fn(i32, i32) -> i32) -> LambdaWrapper;

    #[cpp(func = "std::unique_ptr<StateLambda> _cpp2rust_make_unique_state_lambda_with_initial_value(int)")]
    pub fn state_lambda_new(initial_value: i32) -> StateLambda;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "std::unique_ptr<Comparator> _cpp2rust_make_unique_comparator_with_cmp(int (*)(int, int))")]
    pub unsafe fn comparator_new(cmp: unsafe extern "C" fn(i32, i32) -> i32) -> Comparator;

    #[cpp(func = "std::unique_ptr<Comparator> _cpp2rust_make_unique_comparator_new_add()")]
    pub fn comparator_new_add() -> Comparator;

    #[cpp(func = "std::unique_ptr<LambdaWrapper> _cpp2rust_make_unique_lambda_wrapper_new_add()")]
    pub fn make_add_lambda() -> LambdaWrapper;

    #[cpp(func = "std::unique_ptr<LambdaWrapper> _cpp2rust_make_unique_lambda_wrapper_new_multiply()")]
    pub fn make_multiply_lambda() -> LambdaWrapper;

    #[cpp(func = "std::unique_ptr<LambdaWrapper> _cpp2rust_make_unique_lambda_wrapper_new_max()")]
    pub fn make_max_lambda() -> LambdaWrapper;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "int apply_operation(int, int, int (*)(int, int))")]
    pub unsafe fn apply_operation(a: i32, b: i32, op: unsafe extern "C" fn(i32, i32) -> i32) -> i32;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "int apply_twice(int, int (*)(int, int))")]
    pub unsafe fn apply_twice(x: i32, op: unsafe extern "C" fn(i32, i32) -> i32) -> i32;

    #[cpp(func = "int add_impl(int, int)")]
    pub fn add_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "int multiply_impl(int, int)")]
    pub fn multiply_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "int max_impl(int, int)")]
    pub fn max_impl(a: i32, b: i32) -> i32;
}
