// 此文件为 cpp2rust-demo 工具对 017_virtual_override 自动生成的支架黄金文件，
// 仅供 L1 golden 测试进行生成准确性验证。
//
// Direct 模式下，工具为 Base/Derived 生成 import_class! + make_unique 工厂，
// 使用 std::make_unique<T>(args) 模板函数绑定（无法直接通过 hicc 导出模板，
// 实际 lib.rs 用手动 C++ 包装函数替代）。
hicc::cpp! {
    #include <string>
    #include "virtual_override.h"
}

hicc::import_class! {
    #[cpp(class = "Base")]
    pub class Base {
        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "Derived")]
    pub class Derived {
        #[cpp(method = "const char* getName() const")]
        pub fn get_name(&self) -> *const i8;

        #[cpp(method = "double area() const")]
        pub fn area(&self) -> f64;

        #[cpp(method = "double getValue() const")]
        pub fn get_value(&self) -> f64;
    }
}

hicc::import_lib! {
    #![link_name = "virtual_override"]

    class Base;
    class Derived;

    #[cpp(func = "std::unique_ptr<Base> std::make_unique<Base>(const char*)")]
    pub unsafe fn base_new_with_n(n: *const i8) -> Base;

    #[cpp(func = "std::unique_ptr<Derived> std::make_unique<Derived>(double)")]
    pub fn derived_new_with_v(v: f64) -> Derived;
}
