// 此文件为 cpp2rust-demo 工具对 026_template_specialization 自动生成的支架黄金文件，
// 仅供 L1 golden 测试（test_026_template_specialization）校验工具默认产物的生成准确性。

hicc::cpp! {
    #include <iostream>
    #include <cstring>
    #include <cstdlib>
    #include <cstdio>

    #include "template_specialization.h"
}

hicc::import_class! {
    #[cpp(class = "IntHolder")]
    pub class IntHolder {
        #[cpp(method = "int get() const")]
        pub fn get(&self) -> i32;

        #[cpp(method = "const char* describe() const")]
        pub fn describe(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "DoubleHolder")]
    pub class DoubleHolder {
        #[cpp(method = "double get() const")]
        pub fn get(&self) -> f64;

        #[cpp(method = "const char* describe() const")]
        pub fn describe(&self) -> *const i8;
    }
}

hicc::import_class! {
    #[cpp(class = "StringHolder")]
    pub class StringHolder {
        #[cpp(method = "const char* get() const")]
        pub fn get(&self) -> *const i8;

        #[cpp(method = "const char* describe() const")]
        pub fn describe(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "template_specialization"]

    class IntHolder;
    class DoubleHolder;
    class StringHolder;

    #[cpp(func = "std::unique_ptr<IntHolder> std::make_unique<IntHolder>(int)")]
    pub fn int_holder_new_with_value(value: i32) -> IntHolder;

    #[cpp(func = "std::unique_ptr<DoubleHolder> std::make_unique<DoubleHolder>(double)")]
    pub fn double_holder_new_with_value(value: f64) -> DoubleHolder;

    #[cpp(func = "std::unique_ptr<StringHolder> std::make_unique<StringHolder>(const char*)")]
    pub unsafe fn string_holder_new_with_value(value: *const i8) -> StringHolder;
}
