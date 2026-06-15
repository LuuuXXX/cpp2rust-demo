// 此文件为 cpp2rust-demo 工具对 020_friend_function 自动生成的支架黄金文件，
// 仅供 L1 golden 测试进行生成准确性验证。
//
// Direct 模式下，工具为 MyClass 生成 import_class! 及 friend 函数的 import_lib! 绑定。
// 实际 lib.rs 添加了手动 C++ 包装函数（myclass_new）用于构造 MyClass 对象。
hicc::cpp! {
    #include <iostream>
    #include "friend_function.h"
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

    #[cpp(func = "int friend_function_getSum(const MyClass* a, const MyClass* b)")]
    pub fn friend_function_get_sum(a: *const MyClass, b: *const MyClass) -> i32;

    #[cpp(func = "int friend_function_getProduct(const MyClass* a, const MyClass* b)")]
    pub fn friend_function_get_product(a: *const MyClass, b: *const MyClass) -> i32;

    #[cpp(func = "int friend_function_compare(const MyClass* a, const MyClass* b)")]
    pub fn friend_function_compare(a: *const MyClass, b: *const MyClass) -> i32;
}
