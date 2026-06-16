//! 020_friend_function: 友元函数（命名空间类 + make_unique 工厂）。
//!
//! hicc 直出只绑定类的公有成员方法 `getValue()`/`setValue()` 与构造工厂
//! （见 `lib_scaffold.rs`）。友元函数 `getSum`/`getProduct`/`compare` 是非成员自由
//! 函数（在类体内内联定义，经 ADL 访问私有成员 `value_`），不进 `import_class!`；
//! 本文件用 `hicc::cpp!` 把每个友元包成具名 C++ 函数（经 ADL 调用真实友元），再用
//! `#[cpp(func = ...)]` 绑定为 `MyClass` 的关联方法。

hicc::cpp! {
    #include "friend_function.h"

    using friend_function_ns::MyClass;

    int myclass_friend_sum(const MyClass* self, const MyClass& other) {
        return getSum(*self, other);
    }
    int myclass_friend_product(const MyClass* self, const MyClass& other) {
        return getProduct(*self, other);
    }
    int myclass_friend_compare(const MyClass* self, const MyClass& other) {
        return compare(*self, other);
    }
}

hicc::import_class! {
    #[cpp(class = "friend_function_ns::MyClass")]
    pub class MyClass {
        #[cpp(method = "int getValue() const")]
        pub fn get_value(&self) -> i32;

        #[cpp(method = "void setValue(int v)")]
        pub fn set_value(&mut self, v: i32);

        // 友元函数：经 hicc::cpp! 命名包装绑定为关联方法
        #[cpp(func = "int myclass_friend_sum(const friend_function_ns::MyClass*, const friend_function_ns::MyClass&)")]
        pub fn friend_sum(&self, other: &MyClass) -> i32;

        #[cpp(func = "int myclass_friend_product(const friend_function_ns::MyClass*, const friend_function_ns::MyClass&)")]
        pub fn friend_product(&self, other: &MyClass) -> i32;

        #[cpp(func = "int myclass_friend_compare(const friend_function_ns::MyClass*, const friend_function_ns::MyClass&)")]
        pub fn friend_compare(&self, other: &MyClass) -> i32;

        pub fn new(v: i32) -> Self { my_class_new(v) }
    }
}

hicc::import_lib! {
    #![link_name = "friend_function"]

    #[cpp(func = "std::unique_ptr<friend_function_ns::MyClass> hicc::make_unique<friend_function_ns::MyClass, int>(int&&)")]
    pub fn my_class_new(v: i32) -> MyClass;
}
