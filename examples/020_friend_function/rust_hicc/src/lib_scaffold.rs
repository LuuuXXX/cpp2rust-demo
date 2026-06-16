// 020_friend_function 工具默认产物支架（hicc 直出，去 shim）。
//
// 用于 L1 黄金比对：校验 `init` 对「含友元函数的命名空间类」默认生成的 hicc 骨架。
// 友元函数为非成员自由函数（在类体内内联定义，经 ADL 访问私有成员），hicc 直出不
// 绑定自由函数，故默认支架仅含类方法 getValue/setValue 与构造工厂；完整友元绑定见
// 手写 `lib.rs`。

hicc::cpp! {
    #include "friend_function.h"
}

hicc::import_class! {
    #[cpp(class = "friend_function_ns::MyClass")]
    pub class MyClass {
        #[cpp(method = "int getValue() const")]
        pub fn get_value(&self) -> i32;

        #[cpp(method = "void setValue(int v)")]
        pub fn set_value(&mut self, v: i32);

        pub fn new(v: i32) -> Self { my_class_new(v) }
    }
}

hicc::import_lib! {
    #![link_name = "friend_function"]

    #[cpp(func = "std::unique_ptr<friend_function_ns::MyClass> hicc::make_unique<friend_function_ns::MyClass, int>(int&&)")]
    pub fn my_class_new(v: i32) -> MyClass;
}
