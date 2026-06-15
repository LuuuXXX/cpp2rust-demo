//! 006_class_basic: 类成员函数 + 构造工厂。
//!
//! hicc 模式（参考 hicc-examples/destroy）：
//! - `import_class!` 声明 C++ 类与成员方法
//! - 在 class 内写 `fn new(...) -> Self { factory_fn(...) }`，构造函数体在 `import_lib!` 中

hicc::cpp! {
    #include "class_basic.h"
    #include <hicc/std/string.hpp>
}

hicc::import_class! {
    class string = hicc_std::string;

    #[cpp(class = "class_basic_ns::Counter")]
    pub class Counter {
        #[cpp(method = "void inc()")]
        pub fn inc(&mut self);

        #[cpp(method = "void inc_by(int)")]
        pub fn inc_by(&mut self, delta: i32);

        #[cpp(method = "void reset()")]
        pub fn reset(&mut self);

        #[cpp(method = "int count() const")]
        pub fn count(&self) -> i32;

        // 显式声明返回 ClassRef<'_, string>，借用绑定到 self 的生命周期
        #[cpp(method = "const std::string& name() const")]
        pub fn name(&self) -> &string;

        pub fn new() -> Self {
            counter_new()
        }
        pub fn with_name(name: &string) -> Self {
            counter_with_name(name)
        }
    }
}

hicc::import_lib! {
    #![link_name = "class_basic"]

    #[cpp(func = "std::unique_ptr<class_basic_ns::Counter> hicc::make_unique<class_basic_ns::Counter>()")]
    pub fn counter_new() -> Counter;

    #[cpp(func = "std::unique_ptr<class_basic_ns::Counter> hicc::make_unique<class_basic_ns::Counter, const std::string&>(const std::string&)")]
    #[allow(non_snake_case)]
    pub fn counter_with_name(name: &hicc_std::string) -> Counter;
}
