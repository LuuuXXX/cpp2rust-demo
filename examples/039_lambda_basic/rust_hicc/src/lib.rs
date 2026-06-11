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
        pub fn invoke(&mut self, a: i32, b: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "StateLambda", destroy = "state_lambda_delete")]
    pub class StateLambda {
        #[cpp(method = "int get_value() const")]
        pub fn get_value(&self) -> i32;

        #[cpp(method = "int add(int delta)")]
        pub fn add(&mut self, delta: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Comparator", destroy = "comparator_delete")]
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
    #[cpp(func = "LambdaWrapper* lambda_wrapper_new(int (*)(int, int))")]
    pub unsafe fn lambda_wrapper_new(fn_: unsafe extern "C" fn(i32, i32) -> i32) -> LambdaWrapper;

    #[cpp(func = "StateLambda* state_lambda_new(int)")]
    pub fn state_lambda_new(initial_value: i32) -> StateLambda;

    // cpp2rust-todo[FP]: 含函数指针参数，需确保回调符合 extern "C" 调用约定
    #[cpp(func = "Comparator* comparator_new(int (*)(int, int))")]
    pub unsafe fn comparator_new(cmp: unsafe extern "C" fn(i32, i32) -> i32) -> Comparator;

    #[cpp(func = "Comparator* comparator_new_add()")]
    pub fn comparator_new_add() -> Comparator;

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

    #[cpp(func = "LambdaWrapper* make_add_lambda()")]
    pub fn make_add_lambda() -> *mut LambdaWrapper;

    #[cpp(func = "LambdaWrapper* make_multiply_lambda()")]
    pub fn make_multiply_lambda() -> *mut LambdaWrapper;

    #[cpp(func = "LambdaWrapper* make_max_lambda()")]
    pub fn make_max_lambda() -> *mut LambdaWrapper;
}
