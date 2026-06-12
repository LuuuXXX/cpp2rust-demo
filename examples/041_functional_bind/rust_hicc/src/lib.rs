hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <functional>
    #include <string>

    #include "functional_bind.h"
}

hicc::import_class! {
    #[cpp(class = "Adder", destroy = "adder_delete")]
    pub class Adder {
        #[cpp(method = "int add(int value)")]
        pub fn add(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Multiplier", destroy = "multiplier_delete")]
    pub class Multiplier {
        #[cpp(method = "int multiply(int value)")]
        pub fn multiply(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "StringProcessor", destroy = "string_processor_delete")]
    pub class StringProcessor {
        #[cpp(method = "void set_target(const char* t)")]
        pub fn set_target(&mut self, t: *const i8);

        #[cpp(method = "int count_char(char ch)")]
        pub fn count_char(&mut self, ch: i8) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "functional_bind"]

    class Adder;
    class Multiplier;
    class StringProcessor;

    #[cpp(func = "Adder* adder_new(int)")]
    pub fn adder_new(base_value: i32) -> Adder;

    #[cpp(func = "Multiplier* multiplier_new(int)")]
    pub fn multiplier_new(factor: i32) -> Multiplier;

    #[cpp(func = "StringProcessor* string_processor_new()")]
    pub fn string_processor_new() -> StringProcessor;

    #[cpp(func = "int add_five_impl(int, int)")]
    pub fn add_five_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "int add_ten_impl(int, int)")]
    pub fn add_ten_impl(a: i32, b: i32) -> i32;

    #[cpp(func = "int add_five(int)")]
    pub fn add_five(a: i32) -> i32;

    #[cpp(func = "int add_ten(int)")]
    pub fn add_ten(a: i32) -> i32;
}
