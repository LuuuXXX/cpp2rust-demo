hicc::cpp! {
    #include <stddef.h>
    #include <iostream>
    #include <functional>
    #include <string>
    #include <memory>

    #include "functional_bind.h"

    std::unique_ptr<Adder> adder_new(int base_value) {
        return std::make_unique<Adder>(base_value);
    }
    std::unique_ptr<Multiplier> multiplier_new(int factor) {
        return std::make_unique<Multiplier>(factor);
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

    class Adder;
    class Multiplier;
    class StringProcessor;

    #[cpp(func = "std::unique_ptr<Adder> adder_new(int)")]
    pub fn adder_new(base_value: i32) -> Adder;

    #[cpp(func = "std::unique_ptr<Multiplier> multiplier_new(int)")]
    pub fn multiplier_new(factor: i32) -> Multiplier;

    #[cpp(func = "std::unique_ptr<StringProcessor> hicc::make_unique<StringProcessor>()")]
    pub fn string_processor_new() -> StringProcessor;
}
