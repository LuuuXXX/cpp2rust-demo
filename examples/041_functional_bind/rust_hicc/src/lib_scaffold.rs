hicc::cpp! {
    #include <iostream>
    #include <functional>
    #include <string>

    #include "functional_bind.h"
}

hicc::import_class! {
    #[cpp(class = "AdderImpl")]
    pub class AdderImpl {
        #[cpp(method = "int add(int value)")]
        pub fn add(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "MultiplierImpl")]
    pub class MultiplierImpl {
        #[cpp(method = "int multiply(int value)")]
        pub fn multiply(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "StringProcessorImpl")]
    pub class StringProcessorImpl {
        #[cpp(method = "void set_target(const char* t)")]
        pub fn set_target(&mut self, t: *const i8);

        #[cpp(method = "int count_char(char ch)")]
        pub fn count_char(&mut self, ch: i8) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Adder")]
    pub class Adder {
        #[cpp(method = "int add(int value)")]
        pub fn add(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "Multiplier")]
    pub class Multiplier {
        #[cpp(method = "int multiply(int value)")]
        pub fn multiply(&mut self, value: i32) -> i32;
    }
}

hicc::import_class! {
    #[cpp(class = "StringProcessor")]
    pub class StringProcessor {
        #[cpp(method = "void set_target(const char* t)")]
        pub fn set_target(&mut self, t: *const i8);

        #[cpp(method = "int count_char(char ch)")]
        pub fn count_char(&mut self, ch: i8) -> i32;
    }
}

hicc::import_lib! {
    #![link_name = "functional_bind"]

    class AdderImpl;
    class MultiplierImpl;
    class StringProcessorImpl;
    class Adder;
    class Multiplier;
    class StringProcessor;

    #[cpp(func = "std::unique_ptr<AdderImpl> std::make_unique<AdderImpl>(int)")]
    pub fn adder_impl_new_with_base(base: i32) -> AdderImpl;

    #[cpp(func = "std::unique_ptr<MultiplierImpl> std::make_unique<MultiplierImpl>(int)")]
    pub fn multiplier_impl_new_with_f(f: i32) -> MultiplierImpl;

    #[cpp(func = "std::unique_ptr<StringProcessorImpl> hicc::make_unique<StringProcessorImpl>()")]
    pub fn string_processor_impl_new() -> StringProcessorImpl;

    #[cpp(func = "std::unique_ptr<Adder> std::make_unique<Adder>(int)")]
    pub fn adder_new_with_base_value(base_value: i32) -> Adder;

    #[cpp(func = "std::unique_ptr<Multiplier> std::make_unique<Multiplier>(int)")]
    pub fn multiplier_new_with_factor(factor: i32) -> Multiplier;

    #[cpp(func = "std::unique_ptr<StringProcessor> hicc::make_unique<StringProcessor>()")]
    pub fn string_processor_new() -> StringProcessor;
}
