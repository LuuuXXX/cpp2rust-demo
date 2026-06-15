hicc::cpp! {
    #include <cstddef>
    #include <cstdint>
    #include <iostream>
    #include <cstring>

    #include "union_basic.h"
}

hicc::import_class! {
    #[cpp(class = "Variant")]
    pub class Variant {
        #[cpp(method = "int get_type() const")]
        pub fn get_type(&self) -> i32;

        #[cpp(method = "void set_int(int value)")]
        pub fn set_int(&mut self, value: i32);

        #[cpp(method = "void set_float(float value)")]
        pub fn set_float(&mut self, value: f32);

        #[cpp(method = "void set_string(const char* value)")]
        pub fn set_string(&mut self, value: *const i8);

        #[cpp(method = "int get_int() const")]
        pub fn get_int(&self) -> i32;

        #[cpp(method = "float get_float() const")]
        pub fn get_float(&self) -> f32;

        #[cpp(method = "const char* get_string() const")]
        pub fn get_string(&self) -> *const i8;
    }
}

hicc::import_lib! {
    #![link_name = "union_basic"]

    class Variant;

    #[cpp(func = "std::unique_ptr<Variant> hicc::make_unique<Variant>()")]
    pub fn variant_new() -> Variant;
}
