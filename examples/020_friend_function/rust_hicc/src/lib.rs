hicc::cpp! {
    #include <iostream>

    #include "friend_function.h"
    std::unique_ptr<MyClass> myclass_new(int v) {
        return std::make_unique<MyClass>(v);
    }
    int friend_function_getSum(const MyClass* a, const MyClass* b);
    int friend_function_getProduct(const MyClass* a, const MyClass* b);
    int friend_function_compare(const MyClass* a, const MyClass* b);
}

hicc::import_class! {
    #[cpp(class = "MyClass")]
    pub class MyClass {
        #[cpp(method = "int getValue() const")]
        pub fn get_value(&self) -> i32;

        #[cpp(method = "void setValue(int v)")]
        pub fn set_value(&mut self, v: i32);
    }
}

hicc::import_lib! {
    #![link_name = "friend_function"]

    class MyClass;

    #[cpp(func = "std::unique_ptr<MyClass> myclass_new(int)")]
    pub fn myclass_new_with_v(v: i32) -> MyClass;

    #[cpp(func = "int friend_function_getSum(const MyClass* a, const MyClass* b)")]
    pub fn friend_function_get_sum(a: *const MyClass, b: *const MyClass) -> i32;

    #[cpp(func = "int friend_function_getProduct(const MyClass* a, const MyClass* b)")]
    pub fn friend_function_get_product(a: *const MyClass, b: *const MyClass) -> i32;

    #[cpp(func = "int friend_function_compare(const MyClass* a, const MyClass* b)")]
    pub fn friend_function_compare(a: *const MyClass, b: *const MyClass) -> i32;
}
